use std::collections::HashMap;

use azure_ml::models::JobStatus;
use chrono::{DateTime, Utc};
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
/// 5. Separately fetches pending/scheduled jobs and sends them as a dedicated batch
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

        // Collect IDs of jobs seen in the main search to deduplicate against pending fetch
        let mut seen_ids = std::collections::HashSet::new();

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

                // Track all run IDs we've seen
                for run in &response.runs {
                    seen_ids.insert(run.info.run_id.clone());
                }

                if !rows.is_empty() {
                    let _ = action_tx.send(Action::RecentJobsBatchLoaded(rows.clone()));

                    // Enrich in background, sharing the cancellation token
                    let enrich_client = client.clone();
                    let enrich_tx = action_tx.clone();
                    let enrich_cancel = cancel.clone();
                    let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
                    tokio::spawn(enrich_jobs(enrich_client, ids, enrich_tx, enrich_cancel));
                }
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

        // Step 3: Separately fetch pending/scheduled jobs
        let pending_rows =
            fetch_pending_jobs(&client, &username, &experiment_ids, &exp_names, &cancel).await;

        if cancel.is_cancelled() {
            return;
        }

        // Deduplicate against jobs already seen in the main search
        let new_pending: Vec<RecentJobRow> = pending_rows
            .into_iter()
            .filter(|r| !seen_ids.contains(&r.id))
            .collect();

        if !new_pending.is_empty() {
            let ids: Vec<String> = new_pending.iter().map(|r| r.id.clone()).collect();
            let _ = action_tx.send(Action::RecentJobsPendingBatch(new_pending));

            // Enrich pending jobs too
            let enrich_client = client.clone();
            let enrich_tx = action_tx.clone();
            let enrich_cancel = cancel.clone();
            tokio::spawn(enrich_jobs(enrich_client, ids, enrich_tx, enrich_cancel));
        }

        if !cancel.is_cancelled() {
            let _ = action_tx.send(Action::RecentJobsFetchComplete);
        }
    });
}

/// Fetches jobs that haven't started yet by searching with `start_time ASC` ordering.
/// Jobs without a start_time naturally appear first in this ordering. Stops paginating
/// once all runs on a page have a start_time (meaning no more pending jobs remain).
/// Returns mapped rows without deduplication (caller is responsible for that).
async fn fetch_pending_jobs(
    client: &AzureClient,
    username: &str,
    experiment_ids: &[String],
    exp_names: &HashMap<String, String>,
    cancel: &CancellationToken,
) -> Vec<RecentJobRow> {
    let mut all_rows = Vec::new();
    let filter = format!("tags.mlflow.user='{}'", username);
    let mut page_token: Option<String> = None;

    loop {
        if cancel.is_cancelled() {
            return all_rows;
        }

        let request = SearchRunsRequest {
            experiment_ids: experiment_ids.to_vec(),
            filter: Some(filter.clone()),
            max_results: Some(MLFLOW_PAGE_SIZE),
            order_by: Some(vec!["start_time ASC".to_string()]),
            page_token: page_token.clone(),
        };

        match client.mlflow().search_runs(&request).await {
            Ok(response) => {
                let mut found_started = false;
                for run in &response.runs {
                    let row = map_run_to_row(run, exp_names);
                    if row.start_time.is_some() {
                        // Hit a job with a start_time — all pending jobs have been seen
                        found_started = true;
                        break;
                    }
                    all_rows.push(row);
                }

                // Stop paginating if we hit a started job or there are no more pages
                if found_started {
                    break;
                }

                match response.next_page_token {
                    Some(token) if !token.is_empty() => {
                        page_token = Some(token);
                    }
                    _ => break,
                }
            }
            Err(e) => {
                tracing::debug!("Failed to fetch pending runs: {}", e);
                break;
            }
        }
    }

    all_rows
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
        status: mlflow_status_to_job_status(&run.info.status),
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

/// Maps an MLflow status string to the Azure ML `JobStatus` enum.
pub fn mlflow_status_to_job_status(status: &str) -> JobStatus {
    match status {
        "FINISHED" => JobStatus::Completed,
        "FAILED" => JobStatus::Failed,
        "RUNNING" => JobStatus::Running,
        "KILLED" => JobStatus::Canceled,
        "SCHEDULED" => JobStatus::Queued,
        "STARTING" => JobStatus::Starting,
        _ => JobStatus::Unknown,
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
                            let (compute, jtype, cmd, env, desc, tags, status) = match props {
                                JobBaseProperties::CommandJob(cmd_job) => (
                                    extract_compute(&cmd_job.compute_id),
                                    Some("Command".to_string()),
                                    cmd_job.command.clone(),
                                    cmd_job.environment_id.clone(),
                                    cmd_job.description.clone(),
                                    cmd_job.tags.unwrap_or_default(),
                                    cmd_job.status,
                                ),
                                JobBaseProperties::PipelineJob(pj) => (
                                    extract_compute(&pj.compute_id),
                                    Some("Pipeline".to_string()),
                                    None,
                                    None,
                                    pj.description.clone(),
                                    pj.tags.unwrap_or_default(),
                                    pj.status,
                                ),
                                JobBaseProperties::SweepJob(sj) => (
                                    extract_compute(&sj.compute_id),
                                    Some("Sweep".to_string()),
                                    None,
                                    None,
                                    sj.description.clone(),
                                    sj.tags.unwrap_or_default(),
                                    sj.status,
                                ),
                                _ => (None, None, None, None, None, HashMap::new(), None),
                            };
                            let _ = tx.send(Action::RecentJobEnriched {
                                job_id,
                                compute_target: compute,
                                job_type: jtype,
                                command: cmd,
                                environment_id: env,
                                description: desc,
                                tags,
                                status,
                                end_time: None,
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

/// Spawns an incremental refresh that:
/// 1. Searches for new jobs with start_time > latest_start_time
/// 2. Separately fetches pending/scheduled jobs to catch new ones without start times
/// 3. Re-enriches active (non-terminal) jobs to pick up status changes
/// 4. Refreshes start_time for active jobs that have no start_time yet via MLflow get_run
/// 5. Enriches any new jobs found
pub fn spawn_incremental_refresh(
    client: AzureClient,
    username: String,
    action_tx: ActionSender,
    cancel: CancellationToken,
    known_experiments: HashMap<String, String>,
    latest_start_time: DateTime<Utc>,
    active_job_ids: Vec<String>,
    pending_job_ids: Vec<String>,
) {
    tokio::spawn(async move {
        // Step 1: Check for new experiments (incremental)
        let exp_names =
            match fetch_experiments_incremental(&client, &cancel, &action_tx, known_experiments)
                .await
            {
                Some(map) => map,
                None => return,
            };

        if cancel.is_cancelled() {
            return;
        }

        if !exp_names.is_empty() {
            let _ = action_tx.send(Action::ExperimentCacheUpdated(exp_names.clone()));
        }

        // Step 2: Search for new jobs, sorted by start_time DESC, filter locally
        if !exp_names.is_empty() {
            let filter = format!("tags.mlflow.user='{}'", username);
            let experiment_ids: Vec<String> = exp_names.keys().cloned().collect();

            let request = SearchRunsRequest {
                experiment_ids: experiment_ids.clone(),
                filter: Some(filter),
                max_results: Some(MLFLOW_PAGE_SIZE),
                order_by: Some(vec!["start_time DESC".to_string()]),
                page_token: None,
            };

            match client.mlflow().search_runs(&request).await {
                Ok(response) => {
                    if cancel.is_cancelled() {
                        return;
                    }
                    let mut new_runs = Vec::new();
                    for run in &response.runs {
                        let row = map_run_to_row(run, &exp_names);
                        let is_new = row
                            .start_time
                            .map(|st| st > latest_start_time)
                            .unwrap_or(false);

                        if is_new {
                            new_runs.push(row);
                        } else {
                            // Existing run — send updated metric_keys so the UI
                            // picks up any metrics logged since the initial fetch
                            let metric_keys: Vec<String> =
                                run.data.metrics.iter().map(|m| m.key.clone()).collect();
                            if !metric_keys.is_empty() {
                                let _ = action_tx.send(Action::MetricKeysUpdated {
                                    job_id: row.id,
                                    metric_keys,
                                });
                            }
                        }
                    }

                    if !new_runs.is_empty() {
                        let _ =
                            action_tx.send(Action::RecentJobsIncrementalBatch(new_runs.clone()));

                        // Enrich new jobs
                        let ids: Vec<String> = new_runs.iter().map(|r| r.id.clone()).collect();
                        let enrich_client = client.clone();
                        let enrich_tx = action_tx.clone();
                        let enrich_cancel = cancel.clone();
                        tokio::spawn(enrich_jobs(enrich_client, ids, enrich_tx, enrich_cancel));
                    }
                }
                Err(e) => {
                    if !cancel.is_cancelled() {
                        let _ = action_tx.send(Action::Error(format!(
                            "Failed to search for new runs: {}",
                            e
                        )));
                    }
                }
            }

            if cancel.is_cancelled() {
                return;
            }

            // Step 3: Fetch pending/scheduled jobs to catch new ones without start times
            let pending_rows =
                fetch_pending_jobs(&client, &username, &experiment_ids, &exp_names, &cancel).await;

            if cancel.is_cancelled() {
                return;
            }

            if !pending_rows.is_empty() {
                // Send all pending rows — the tab will deduplicate against existing jobs
                let ids: Vec<String> = pending_rows.iter().map(|r| r.id.clone()).collect();
                let _ = action_tx.send(Action::RecentJobsPendingBatch(pending_rows));

                // Enrich pending jobs
                let enrich_client = client.clone();
                let enrich_tx = action_tx.clone();
                let enrich_cancel = cancel.clone();
                tokio::spawn(enrich_jobs(enrich_client, ids, enrich_tx, enrich_cancel));
            }
        }

        if cancel.is_cancelled() {
            return;
        }

        // Step 4: Re-enrich active jobs to pick up status changes
        if !active_job_ids.is_empty() {
            enrich_jobs(
                client.clone(),
                active_job_ids,
                action_tx.clone(),
                cancel.clone(),
            )
            .await;
        }

        if cancel.is_cancelled() {
            return;
        }

        // Step 5: Refresh start_time for jobs that were pending and may have started
        if !pending_job_ids.is_empty() {
            refresh_start_times(
                client.clone(),
                pending_job_ids,
                action_tx.clone(),
                cancel.clone(),
            )
            .await;
        }

        if !cancel.is_cancelled() {
            let _ = action_tx.send(Action::RecentJobsFetchComplete);
        }
    });
}

/// For jobs with missing start_time, calls MLflow get_run to check if they now have one.
/// Sends `RecentJobStartTimeUpdated` actions for any that do.
async fn refresh_start_times(
    client: AzureClient,
    job_ids: Vec<String>,
    action_tx: ActionSender,
    cancel: CancellationToken,
) {
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
        let client = client.clone();
        let tx = action_tx.clone();
        let cancel = cancel.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            if cancel.is_cancelled() {
                return;
            }
            match client.mlflow().get_run(&id).await {
                Ok(run) => {
                    if cancel.is_cancelled() {
                        return;
                    }
                    if let Some(start_time) = run
                        .info
                        .start_time
                        .as_deref()
                        .and_then(|s| s.parse::<i64>().ok())
                        .and_then(DateTime::from_timestamp_millis)
                    {
                        let _ = tx.send(Action::RecentJobStartTimeUpdated {
                            job_id: id,
                            start_time,
                        });
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to refresh start_time for job {}: {}", id, e);
                }
            }
        }));
    }

    futures::future::join_all(handles).await;
}
