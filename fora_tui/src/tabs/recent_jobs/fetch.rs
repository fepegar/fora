use std::collections::HashMap;

use chrono::DateTime;
use mlflow::{Run, SearchRunsRequest};
use tokio_util::sync::CancellationToken;

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::ActionSender;

use super::state::RecentJobRow;

/// How many runs to request per MLflow page.
pub const MLFLOW_PAGE_SIZE: u32 = 1000;
/// How many jobs to enrich concurrently.
pub const ENRICH_CONCURRENCY: usize = 20;

/// Spawns a background task that:
/// 1. Fetches experiment IDs via MLflow (incrementally if `known_experiments` is non-empty)
/// 2. Searches runs filtered by username, ordered by start_time DESC
/// 3. Sends batches of rows to the UI
/// 4. Enriches each row via Azure ML jobs.get() in background
///
/// The task checks `cancel` between pages and before sending actions,
/// exiting early if cancellation has been requested.
///
/// If `known_experiments` is non-empty, experiments are fetched incrementally by
/// `creation_time DESC`, stopping once a full page of already-known IDs is seen.
pub fn spawn_recent_jobs_fetcher(
    client: AzureClient,
    username: String,
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
                None => return, // cancelled or error already reported
            };

        if cancel.is_cancelled() {
            return;
        }

        if exp_names.is_empty() {
            let _ = action_tx.send(Action::RecentJobsFetchComplete);
            return;
        }

        // Notify the tab of the updated experiment cache
        let _ = action_tx.send(Action::ExperimentCacheUpdated(exp_names.clone()));

        let experiment_ids: Vec<String> = exp_names.keys().cloned().collect();

        // Step 2: Search runs with user filter, paginating
        let filter = format!("tags.mlflow.user='{}'", username);
        let mut page_token: Option<String> = None;

        loop {
            if cancel.is_cancelled() {
                return;
            }

            let request = SearchRunsRequest {
                experiment_ids: experiment_ids.clone(),
                filter: Some(filter.clone()),
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
                    let _ = action_tx.send(Action::Error(format!("Failed to search runs: {}", e)));
                    break;
                }
            };

            if cancel.is_cancelled() {
                return;
            }

            if !response.runs.is_empty() {
                let rows: Vec<RecentJobRow> = response
                    .runs
                    .iter()
                    .map(|run| map_run_to_row(run, &exp_names))
                    .collect();

                let _ = action_tx.send(Action::RecentJobsBatchLoaded(rows.clone()));

                // Enrich in background, sharing the cancellation token
                let enrich_client = client.clone();
                let enrich_tx = action_tx.clone();
                let enrich_cancel = cancel.clone();
                let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
                tokio::spawn(enrich_jobs(enrich_client, ids, enrich_tx, enrich_cancel));
            }

            match response.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }

        if !cancel.is_cancelled() {
            let _ = action_tx.send(Action::RecentJobsFetchComplete);
        }
    });
}

/// Fetches experiments incrementally. If `known` is non-empty, fetches by
/// `creation_time DESC` and stops once a full page of already-known IDs is seen.
/// Returns the merged map of all experiment IDs → names, or `None` on cancellation/error.
pub async fn fetch_experiments_incremental(
    client: &AzureClient,
    cancel: &CancellationToken,
    action_tx: &ActionSender,
    known: HashMap<String, String>,
) -> Option<HashMap<String, String>> {
    let mut exp_names = known;

    if exp_names.is_empty() {
        // First load: fetch all experiments (no ordering optimisation possible)
        let experiments = match client.mlflow().search_all_experiments().await {
            Ok(exps) => exps,
            Err(e) => {
                if cancel.is_cancelled() {
                    return None;
                }
                let _ =
                    action_tx.send(Action::Error(format!("Failed to fetch experiments: {}", e)));
                return None;
            }
        };
        for exp in experiments {
            exp_names.insert(exp.experiment_id, exp.name);
        }
    } else {
        // Incremental: fetch newest experiments first, stop when we see a known page
        let mut page_token: Option<String> = None;

        loop {
            if cancel.is_cancelled() {
                return None;
            }

            let resp = match client
                .mlflow()
                .search_experiments(
                    Some(1000),
                    page_token.as_deref(),
                    Some(vec!["creation_time DESC".to_string()]),
                    None,
                )
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    if cancel.is_cancelled() {
                        return None;
                    }
                    let _ = action_tx
                        .send(Action::Error(format!("Failed to fetch experiments: {}", e)));
                    return None;
                }
            };

            let mut all_known = true;
            for exp in &resp.experiments {
                if !exp_names.contains_key(&exp.experiment_id) {
                    exp_names.insert(exp.experiment_id.clone(), exp.name.clone());
                    all_known = false;
                }
            }

            // If every experiment on this page was already cached, we're done
            if all_known && !resp.experiments.is_empty() {
                break;
            }

            match resp.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }
    }

    Some(exp_names)
}

fn map_run_to_row(run: &Run, exp_names: &HashMap<String, String>) -> RecentJobRow {
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

    let user = run.info.user_id.clone();

    let metric_keys: Vec<String> = run.data.metrics.iter().map(|m| m.key.clone()).collect();

    RecentJobRow {
        id: run.info.run_id.clone(),
        display_name: run_name,
        experiment_name,
        experiment_id: run.info.experiment_id.clone(),
        status: run.info.status.clone(),
        start_time,
        end_time,
        user,
        compute_target: None,
        job_type: None,
        command: None,
        environment_id: None,
        description: None,
        tags: HashMap::new(),
        enriched: false,
        metric_keys,
    }
}

/// Enriches jobs by fetching full details from the Azure ML REST API.
pub async fn enrich_jobs(
    client: AzureClient,
    job_ids: Vec<String>,
    action_tx: ActionSender,
    cancel: CancellationToken,
) {
    use azure_ml::models::JobBaseProperties;

    let rg = client.resource_group().to_string();
    let ws = client.workspace_name().to_string();

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(ENRICH_CONCURRENCY));
    let mut handles = Vec::new();

    for id in job_ids {
        if cancel.is_cancelled() {
            break;
        }
        let permit = semaphore.clone().acquire_owned().await;
        if permit.is_err() {
            break;
        }
        let permit = permit.unwrap();
        let enrich_client = client.clone();
        let rg = rg.clone();
        let ws = ws.clone();
        let tx = action_tx.clone();
        let cancel = cancel.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            if cancel.is_cancelled() {
                return;
            }
            let jc = enrich_client.jobs();
            match jc.get(&rg, &ws, &id, None).await {
                Ok(response) => {
                    if cancel.is_cancelled() {
                        return;
                    }
                    if let Ok(job_base) = response.into_model() {
                        let job_id = job_base.name.unwrap_or_default();
                        if let Some(props) = job_base.properties {
                            let (compute, jtype, cmd, env, desc, tags) = match props {
                                JobBaseProperties::CommandJob(cmd_job) => (
                                    extract_compute(&cmd_job.compute_id),
                                    Some("Command".to_string()),
                                    cmd_job.command.clone(),
                                    cmd_job.environment_id.clone(),
                                    cmd_job.description.clone(),
                                    cmd_job.tags.unwrap_or_default(),
                                ),
                                JobBaseProperties::PipelineJob(pj) => (
                                    extract_compute(&pj.compute_id),
                                    Some("Pipeline".to_string()),
                                    None,
                                    None,
                                    pj.description.clone(),
                                    pj.tags.unwrap_or_default(),
                                ),
                                JobBaseProperties::SweepJob(sj) => (
                                    extract_compute(&sj.compute_id),
                                    Some("Sweep".to_string()),
                                    None,
                                    None,
                                    sj.description.clone(),
                                    sj.tags.unwrap_or_default(),
                                ),
                                _ => (None, None, None, None, None, HashMap::new()),
                            };
                            let _ = tx.send(Action::RecentJobEnriched {
                                job_id,
                                compute_target: compute,
                                job_type: jtype,
                                command: cmd,
                                environment_id: env,
                                description: desc,
                                tags,
                            });
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to enrich job {}: {}", id, e);
                }
            }
        }));
    }

    futures::future::join_all(handles).await;
}

fn extract_compute(compute_id: &Option<String>) -> Option<String> {
    compute_id
        .as_deref()
        .and_then(|id| id.rsplit('/').next())
        .map(|s| s.to_string())
}
