use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use mlflow::SearchRunsRequest;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::recent_jobs::fetch::{fetch_experiments_incremental, mlflow_status_to_job_status};
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::tabs::ActionSender;

/// How many runs to request per MLflow page when loading all jobs for an experiment.
pub const MLFLOW_PAGE_SIZE: u32 = 1000;

/// Maximum concurrent API calls when probing experiments for their latest job.
const DISCOVERY_CONCURRENCY: usize = 200;

/// Spawns a background task that discovers experiments by probing each one in parallel.
///
/// Strategy:
/// 1. Fetch the full experiment list (incrementally when possible).
/// 2. For each experiment, make a parallel `search_runs` call with `max_results=1`
///    and `order_by=["start_time DESC"]` to find the most recent job time.
/// 3. Wait for all probes to complete, then send the full batch to the UI so it
///    can sort and display the complete list at once (no progressive reordering).
///
/// Concurrency is limited by a semaphore to avoid overwhelming the API.
pub fn spawn_experiment_discovery(
    client: AzureClient,
    action_tx: ActionSender,
    cancel: CancellationToken,
    known_experiments: HashMap<String, String>,
) {
    tokio::spawn(async move {
        // Step 1: Fetch experiments (incrementally when possible)
        let exp_names =
            match fetch_experiments_incremental(&client, &cancel, &action_tx, known_experiments)
                .await
            {
                Some(map) => map,
                None => {
                    if !cancel.is_cancelled() {
                        let _ = action_tx.send(Action::ExperimentDiscoveryComplete);
                    }
                    return;
                }
            };

        if cancel.is_cancelled() {
            return;
        }

        if exp_names.is_empty() {
            let _ = action_tx.send(Action::ExperimentDiscoveryComplete);
            return;
        }

        // Notify the tab of the updated experiment cache
        let _ = action_tx.send(Action::ExperimentCacheUpdated(exp_names.clone()));

        // Step 2: Probe each experiment in parallel to find its most recent job time
        let semaphore = Arc::new(Semaphore::new(DISCOVERY_CONCURRENCY));
        let mut handles = Vec::new();

        for (exp_id, exp_name) in &exp_names {
            let client = client.clone();
            let cancel = cancel.clone();
            let semaphore = semaphore.clone();
            let exp_id = exp_id.clone();
            let exp_name = exp_name.clone();

            let handle = tokio::spawn(async move {
                if cancel.is_cancelled() {
                    return None;
                }

                let _permit = match semaphore.acquire().await {
                    Ok(p) => p,
                    Err(_) => return None, // semaphore closed
                };

                if cancel.is_cancelled() {
                    return None;
                }

                let request = SearchRunsRequest {
                    experiment_ids: vec![exp_id.clone()],
                    filter: None,
                    max_results: Some(1),
                    order_by: Some(vec!["start_time DESC".to_string()]),
                    page_token: None,
                };

                let most_recent_time = match client.mlflow().search_runs(&request).await {
                    Ok(response) => response.runs.first().and_then(|run| {
                        run.info
                            .start_time
                            .as_deref()
                            .and_then(|s| s.parse::<i64>().ok())
                            .and_then(DateTime::from_timestamp_millis)
                    }),
                    Err(e) => {
                        tracing::debug!("Failed to probe experiment {}: {}", exp_id, e);
                        None
                    }
                };

                Some((exp_id, exp_name, most_recent_time))
            });

            handles.push(handle);
        }

        // Collect all results, sending progress updates as probes complete
        let total = handles.len();
        let mut discovered = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            if cancel.is_cancelled() {
                return;
            }
            match handle.await {
                Ok(Some(result)) => discovered.push(result),
                Ok(None) => {} // cancelled or semaphore closed
                Err(e) => {
                    tracing::debug!("Experiment probe task panicked: {}", e);
                }
            }
            let _ = action_tx.send(Action::ExperimentDiscoveryProgress {
                completed: i + 1,
                total,
            });
        }

        if cancel.is_cancelled() {
            return;
        }

        if !discovered.is_empty() {
            let _ = action_tx.send(Action::ExperimentsDiscovered(discovered));
        }

        let _ = action_tx.send(Action::ExperimentDiscoveryComplete);
    });
}

/// Spawns a background task to load jobs for a specific experiment.
pub fn spawn_experiment_jobs_fetch(
    client: AzureClient,
    experiment_id: String,
    experiment_name: String,
    action_tx: ActionSender,
    cancel: CancellationToken,
) {
    tokio::spawn(async move {
        let exp_names: HashMap<String, String> = [(experiment_id.clone(), experiment_name)]
            .into_iter()
            .collect();

        let mut all_jobs = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            if cancel.is_cancelled() {
                return;
            }

            let request = SearchRunsRequest {
                experiment_ids: vec![experiment_id.clone()],
                filter: None,
                max_results: Some(MLFLOW_PAGE_SIZE),
                order_by: Some(vec!["start_time DESC".to_string()]),
                page_token: page_token.clone(),
            };

            let response = match client.mlflow().search_runs(&request).await {
                Ok(r) => r,
                Err(e) => {
                    if cancel.is_cancelled() {
                        return;
                    }
                    let _ = action_tx.send(Action::Error(format!(
                        "Failed to fetch jobs for experiment: {}",
                        e
                    )));
                    break;
                }
            };

            for run in &response.runs {
                all_jobs.push(map_run_to_row(run, &exp_names));
            }

            match response.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }

        if cancel.is_cancelled() {
            return;
        }

        let _ = action_tx.send(Action::ExperimentJobsLoaded {
            experiment_id,
            jobs: all_jobs.clone(),
        });

        // Enrich jobs with full details from Azure ML REST API
        let ids: Vec<String> = all_jobs.iter().map(|r| r.id.clone()).collect();
        if !ids.is_empty() {
            crate::tabs::recent_jobs::fetch::enrich_jobs(client, ids, action_tx, cancel).await;
        }
    });
}

fn map_run_to_row(run: &mlflow::Run, exp_names: &HashMap<String, String>) -> RecentJobRow {
    let run_name = run
        .info
        .run_name
        .clone()
        .unwrap_or_else(|| run.info.run_id.clone());

    let experiment_name = exp_names
        .get(&run.info.experiment_id)
        .cloned()
        .unwrap_or_else(|| run.info.experiment_id.clone());

    let start_time = run
        .info
        .start_time
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .and_then(DateTime::from_timestamp_millis);

    let end_time = run
        .info
        .end_time
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok())
        .and_then(DateTime::from_timestamp_millis);

    let metric_keys: Vec<String> = run.data.metrics.iter().map(|m| m.key.clone()).collect();

    RecentJobRow {
        id: run.info.run_id.clone(),
        display_name: run_name,
        experiment_name,
        experiment_id: run.info.experiment_id.clone(),
        status: mlflow_status_to_job_status(&run.info.status),
        start_time,
        end_time,
        user: run.info.user_id.clone(),
        compute_target: None,
        job_type: None,
        command: None,
        environment_id: None,
        description: None,
        tags: std::collections::HashMap::new(),
        enriched: false,
        metric_keys,
    }
}

/// Spawns an incremental experiment refresh that:
/// 1. Fetches new experiments (incrementally)
/// 2. Searches for recent runs since latest_start_time across all experiments
/// 3. Groups by experiment_id to find updated most_recent_job_time
/// 4. Sends updates to the UI
pub fn spawn_incremental_experiment_refresh(
    client: AzureClient,
    action_tx: ActionSender,
    cancel: CancellationToken,
    known_experiments: HashMap<String, String>,
    latest_start_time: DateTime<Utc>,
) {
    tokio::spawn(async move {
        // Step 1: Fetch experiments incrementally (discover new ones)
        let exp_names =
            match fetch_experiments_incremental(&client, &cancel, &action_tx, known_experiments)
                .await
            {
                Some(map) => map,
                None => {
                    if !cancel.is_cancelled() {
                        let _ = action_tx.send(Action::ExperimentIncrementalComplete);
                    }
                    return;
                }
            };

        if cancel.is_cancelled() {
            return;
        }

        if exp_names.is_empty() {
            let _ = action_tx.send(Action::ExperimentIncrementalComplete);
            return;
        }

        // Notify the tab of the updated experiment cache
        let _ = action_tx.send(Action::ExperimentCacheUpdated(exp_names.clone()));

        // Step 2: Search for recent runs sorted by start_time DESC, filter locally
        let experiment_ids: Vec<String> = exp_names.keys().cloned().collect();

        let request = SearchRunsRequest {
            experiment_ids,
            filter: None,
            max_results: Some(MLFLOW_PAGE_SIZE),
            order_by: Some(vec!["start_time DESC".to_string()]),
            page_token: None,
        };

        match client.mlflow().search_runs(&request).await {
            Ok(response) => {
                if cancel.is_cancelled() {
                    return;
                }

                // Group runs newer than latest by experiment_id, find max start_time
                let mut experiment_updates: HashMap<String, Option<DateTime<Utc>>> = HashMap::new();

                for run in &response.runs {
                    let start_time = run
                        .info
                        .start_time
                        .as_deref()
                        .and_then(|s| s.parse::<i64>().ok())
                        .and_then(DateTime::from_timestamp_millis);

                    // Only consider runs newer than our latest known time
                    let is_new = start_time.map(|st| st > latest_start_time).unwrap_or(false);
                    if is_new {
                        let entry = experiment_updates
                            .entry(run.info.experiment_id.clone())
                            .or_insert(None);
                        if let Some(st) = start_time {
                            *entry = Some(entry.map(|cur| cur.max(st)).unwrap_or(st));
                        }
                    } else {
                        // Existing run — send updated metric_keys so the UI
                        // picks up any metrics logged since the initial fetch
                        let metric_keys: Vec<String> =
                            run.data.metrics.iter().map(|m| m.key.clone()).collect();
                        if !metric_keys.is_empty() {
                            let _ = action_tx.send(Action::MetricKeysUpdated {
                                job_id: run.info.run_id.clone(),
                                metric_keys,
                            });
                        }
                    }
                }

                if !experiment_updates.is_empty() {
                    let updates: Vec<(String, String, Option<DateTime<Utc>>)> = experiment_updates
                        .into_iter()
                        .map(|(exp_id, max_time)| {
                            let exp_name = exp_names
                                .get(&exp_id)
                                .cloned()
                                .unwrap_or_else(|| exp_id.clone());
                            (exp_id, exp_name, max_time)
                        })
                        .collect();
                    let _ = action_tx.send(Action::ExperimentIncrementalUpdate(updates));
                }
            }
            Err(e) => {
                if !cancel.is_cancelled() {
                    let _ = action_tx.send(Action::Error(format!(
                        "Failed to search for recent runs: {}",
                        e
                    )));
                }
            }
        }

        if !cancel.is_cancelled() {
            let _ = action_tx.send(Action::ExperimentIncrementalComplete);
        }
    });
}
