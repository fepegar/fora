use std::collections::HashMap;

use chrono::DateTime;
use mlflow::{Run, SearchRunsRequest};

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::ActionSender;

use super::state::RecentJobRow;

/// How many runs to request per MLflow page.
pub const MLFLOW_PAGE_SIZE: u32 = 1000;
/// How many jobs to enrich concurrently.
pub const ENRICH_CONCURRENCY: usize = 20;

/// Spawns a background task that:
/// 1. Fetches all experiment IDs via MLflow
/// 2. Searches runs filtered by username, ordered by start_time DESC
/// 3. Sends batches of rows to the UI
/// 4. Enriches each row via Azure ML jobs.get() in background
pub fn spawn_recent_jobs_fetcher(client: AzureClient, username: String, action_tx: ActionSender) {
    tokio::spawn(async move {
        // Step 1: Get all experiment IDs
        let experiments = match client.mlflow().search_all_experiments().await {
            Ok(exps) => exps,
            Err(e) => {
                let _ =
                    action_tx.send(Action::Error(format!("Failed to fetch experiments: {}", e)));
                let _ = action_tx.send(Action::RecentJobsFetchComplete);
                return;
            }
        };

        if experiments.is_empty() {
            let _ = action_tx.send(Action::RecentJobsFetchComplete);
            return;
        }

        // Build experiment name lookup
        let exp_names: HashMap<String, String> = experiments
            .iter()
            .map(|e| (e.experiment_id.clone(), e.name.clone()))
            .collect();

        let experiment_ids: Vec<String> = experiments
            .iter()
            .map(|e| e.experiment_id.clone())
            .collect();

        // Step 2: Search runs with user filter, paginating
        let filter = format!("tags.mlflow.user='{}'", username);
        let mut page_token: Option<String> = None;

        loop {
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
                    let _ = action_tx.send(Action::Error(format!("Failed to search runs: {}", e)));
                    break;
                }
            };

            if !response.runs.is_empty() {
                let rows: Vec<RecentJobRow> = response
                    .runs
                    .iter()
                    .map(|run| map_run_to_row(run, &exp_names))
                    .collect();

                // Send to UI immediately
                let _ = action_tx.send(Action::RecentJobsBatchLoaded(rows.clone()));

                // Enrich in background
                let enrich_client = client.clone();
                let enrich_tx = action_tx.clone();
                let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
                tokio::spawn(enrich_jobs(enrich_client, ids, enrich_tx));
            }

            match response.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }

        let _ = action_tx.send(Action::RecentJobsFetchComplete);
    });
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
    }
}

/// Enriches jobs by fetching full details from the Azure ML REST API.
pub async fn enrich_jobs(client: AzureClient, job_ids: Vec<String>, action_tx: ActionSender) {
    use azure_ml::models::JobBaseProperties;

    let rg = client.resource_group().to_string();
    let ws = client.workspace_name().to_string();

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(ENRICH_CONCURRENCY));
    let mut handles = Vec::new();

    for id in job_ids {
        let permit = semaphore.clone().acquire_owned().await;
        if permit.is_err() {
            break;
        }
        let permit = permit.unwrap();
        let enrich_client = client.clone();
        let rg = rg.clone();
        let ws = ws.clone();
        let tx = action_tx.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let jc = enrich_client.jobs();
            match jc.get(&rg, &ws, &id, None).await {
                Ok(response) => {
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
