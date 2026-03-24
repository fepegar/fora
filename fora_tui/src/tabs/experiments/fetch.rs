use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mlflow::SearchRunsRequest;
use tokio_util::sync::CancellationToken;

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::recent_jobs::fetch::fetch_experiments_incremental;
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::tabs::ActionSender;

/// How many runs to request per MLflow page for discovery.
pub const MLFLOW_PAGE_SIZE: u32 = 1000;

/// Spawns a background task that discovers experiments ordered by most recent job.
///
/// Strategy: search all runs across all experiments ordered by start_time DESC.
/// As runs come in, we note the first time each experiment_id appears — that's its
/// "most recent job time" and determines sort order. We send experiment discovery
/// events to the UI progressively.
///
/// The task checks `cancel` between pages and before sending actions,
/// exiting early if cancellation has been requested.
///
/// Uses incremental experiment fetching when `known_experiments` is non-empty.
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

        let experiment_ids: Vec<String> = exp_names.keys().cloned().collect();

        // Step 2: Search all runs ordered by start_time DESC
        let mut page_token: Option<String> = None;
        let mut seen_experiments: HashMap<String, DateTime<Utc>> = HashMap::new();

        loop {
            if cancel.is_cancelled() {
                return;
            }

            let request = SearchRunsRequest {
                experiment_ids: experiment_ids.clone(),
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
                        "Failed to search runs for experiments: {}",
                        e
                    )));
                    break;
                }
            };

            if cancel.is_cancelled() {
                return;
            }

            // Process runs: discover experiments in order
            let mut new_experiments = Vec::new();

            for run in &response.runs {
                let exp_id = &run.info.experiment_id;
                if seen_experiments.contains_key(exp_id) {
                    continue;
                }

                let start_time = run
                    .info
                    .start_time
                    .as_deref()
                    .and_then(|s| s.parse::<i64>().ok())
                    .and_then(DateTime::from_timestamp_millis);

                let exp_name = exp_names
                    .get(exp_id)
                    .cloned()
                    .unwrap_or_else(|| exp_id.clone());

                if let Some(t) = start_time {
                    seen_experiments.insert(exp_id.clone(), t);
                }

                new_experiments.push((exp_id.clone(), exp_name, start_time));
            }

            if !new_experiments.is_empty() {
                let _ = action_tx.send(Action::ExperimentsDiscovered(new_experiments));
            }

            match response.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }

        if !cancel.is_cancelled() {
            let _ = action_tx.send(Action::ExperimentDiscoveryComplete);
        }
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
        status: run.info.status.clone(),
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
