use azure_ml::models::{JobBase, JobBaseProperties, JobStatus};
use futures::StreamExt;
use std::collections::HashSet;
use tokio::sync::mpsc;

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::ActionSender;

use super::state::JobRow;

/// How many items to send per UI update.
pub const BATCH_SIZE: usize = 50;
/// How many items to load on the initial fetch.
pub const INITIAL_LOAD: usize = 200;
/// How many items to load on each "load more" request.
pub const PREFETCH_LOAD: usize = 100;
/// Trigger prefetch when selection is this close to the end of loaded data.
pub const PREFETCH_THRESHOLD: usize = 50;

/// Handle to a running fetcher task. Drop to cancel.
pub struct JobFetcherHandle {
    load_more_tx: mpsc::UnboundedSender<usize>,
}

impl JobFetcherHandle {
    /// Ask the fetcher to load `count` more items.
    pub fn load_more(&self, count: usize) {
        let _ = self.load_more_tx.send(count);
    }
}

/// Spawns a background task that owns the Pager and streams items to the UI.
///
/// Returns a handle for requesting more items. Dropping the handle cancels the task.
/// The task immediately begins loading `initial_count` items.
pub fn spawn_job_fetcher(
    client: AzureClient,
    action_tx: ActionSender,
    initial_count: usize,
) -> JobFetcherHandle {
    let (load_more_tx, load_more_rx) = mpsc::unbounded_channel();

    // Send the initial load request
    let _ = load_more_tx.send(initial_count);

    tokio::spawn(job_fetcher_task(client, action_tx, load_more_rx));

    JobFetcherHandle { load_more_tx }
}

async fn job_fetcher_task(
    client: AzureClient,
    action_tx: ActionSender,
    mut load_more_rx: mpsc::UnboundedReceiver<usize>,
) {
    let jobs_client = client.jobs();
    let mut pager = match jobs_client.list(client.resource_group(), client.workspace_name(), None) {
        Ok(p) => p,
        Err(e) => {
            let _ = action_tx.send(Action::Error(format!("Failed to list jobs: {}", e)));
            return;
        }
    };

    // Wait for "load N items" requests
    while let Some(items_to_load) = load_more_rx.recv().await {
        let mut loaded = 0;
        let mut batch = Vec::new();
        let mut exhausted = false;

        while loaded < items_to_load {
            match pager.next().await {
                Some(Ok(job_base)) => {
                    if let Some(row) = map_job_base(job_base) {
                        batch.push(row);
                    }
                    loaded += 1;

                    // Send batch when it reaches BATCH_SIZE
                    if batch.len() >= BATCH_SIZE {
                        let _ = action_tx.send(Action::JobsBatchLoaded(std::mem::take(&mut batch)));
                    }
                }
                Some(Err(e)) => {
                    // Send any partial batch before reporting error
                    if !batch.is_empty() {
                        let _ = action_tx.send(Action::JobsBatchLoaded(std::mem::take(&mut batch)));
                    }
                    let _ = action_tx.send(Action::Error(format!("Failed to fetch jobs: {}", e)));
                    return;
                }
                None => {
                    exhausted = true;
                    break;
                }
            }
        }

        // Send any remaining items
        if !batch.is_empty() {
            let _ = action_tx.send(Action::JobsBatchLoaded(batch));
        }

        if exhausted {
            let _ = action_tx.send(Action::JobsFetchComplete);
            return;
        }

        let _ = action_tx.send(Action::JobsFetchPaused);
        // Loop back and wait for the next "load more" request
    }
}

fn map_job_base(job: JobBase) -> Option<JobRow> {
    let id = job.name.unwrap_or_default();
    let created_at = job
        .system_data
        .as_ref()
        .and_then(|sd| sd.created_at)
        .map(|t| {
            chrono::DateTime::from_timestamp(t.unix_timestamp(), t.nanosecond()).unwrap_or_default()
        });

    let props = job.properties?;
    match props {
        JobBaseProperties::CommandJob(cmd) => Some(JobRow {
            id,
            display_name: cmd.display_name.unwrap_or_default(),
            experiment_name: cmd.experiment_name.unwrap_or_else(|| "Default".to_string()),
            status: cmd.status.unwrap_or(JobStatus::Unknown),
            compute_target: extract_compute_name(&cmd.compute_id),
            created_at,
            job_type: "Command".to_string(),
            command: cmd.command,
            environment_id: cmd.environment_id,
            description: cmd.description,
            tags: cmd.tags.unwrap_or_default(),
            properties: flatten_properties(cmd.properties),
        }),
        JobBaseProperties::PipelineJob(pj) => Some(JobRow {
            id,
            display_name: pj.display_name.unwrap_or_default(),
            experiment_name: pj.experiment_name.unwrap_or_else(|| "Default".to_string()),
            status: pj.status.unwrap_or(JobStatus::Unknown),
            compute_target: extract_compute_name(&pj.compute_id),
            created_at,
            job_type: "Pipeline".to_string(),
            command: None,
            environment_id: None,
            description: pj.description,
            tags: pj.tags.unwrap_or_default(),
            properties: flatten_properties(pj.properties),
        }),
        JobBaseProperties::SweepJob(sj) => Some(JobRow {
            id,
            display_name: sj.display_name.unwrap_or_default(),
            experiment_name: sj.experiment_name.unwrap_or_else(|| "Default".to_string()),
            status: sj.status.unwrap_or(JobStatus::Unknown),
            compute_target: extract_compute_name(&sj.compute_id),
            created_at,
            job_type: "Sweep".to_string(),
            command: None,
            environment_id: None,
            description: sj.description,
            tags: sj.tags.unwrap_or_default(),
            properties: flatten_properties(sj.properties),
        }),
        JobBaseProperties::AutoMLJob(aj) => Some(JobRow {
            id,
            display_name: aj.display_name.unwrap_or_default(),
            experiment_name: aj.experiment_name.unwrap_or_else(|| "Default".to_string()),
            status: aj.status.unwrap_or(JobStatus::Unknown),
            compute_target: extract_compute_name(&aj.compute_id),
            created_at,
            job_type: "AutoML".to_string(),
            command: None,
            environment_id: None,
            description: aj.description,
            tags: aj.tags.unwrap_or_default(),
            properties: flatten_properties(aj.properties),
        }),
        JobBaseProperties::SparkJob(sj) => Some(JobRow {
            id,
            display_name: sj.display_name.unwrap_or_default(),
            experiment_name: sj.experiment_name.unwrap_or_else(|| "Default".to_string()),
            status: sj.status.unwrap_or(JobStatus::Unknown),
            compute_target: extract_compute_name(&sj.compute_id),
            created_at,
            job_type: "Spark".to_string(),
            command: None,
            environment_id: None,
            description: sj.description,
            tags: sj.tags.unwrap_or_default(),
            properties: flatten_properties(sj.properties),
        }),
        JobBaseProperties::UnknownJobType {
            display_name,
            experiment_name,
            status,
            compute_id,
            description,
            tags,
            properties,
            ..
        } => Some(JobRow {
            id,
            display_name: display_name.unwrap_or_default(),
            experiment_name: experiment_name.unwrap_or_else(|| "Default".to_string()),
            status: status.unwrap_or(JobStatus::Unknown),
            compute_target: extract_compute_name(&compute_id),
            created_at,
            job_type: "Unknown".to_string(),
            command: None,
            environment_id: None,
            description,
            tags: tags.unwrap_or_default(),
            properties: properties
                .unwrap_or_default()
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect(),
        }),
    }
}

fn extract_compute_name(compute_id: &Option<String>) -> String {
    compute_id
        .as_deref()
        .and_then(|id| id.rsplit('/').next())
        .unwrap_or("")
        .to_string()
}

fn flatten_properties(
    props: Option<std::collections::HashMap<String, Option<String>>>,
) -> std::collections::HashMap<String, String> {
    props
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(k, v)| v.map(|val| (k, val)))
        .collect()
}

/// Spawns a background task to refresh specific jobs in-place by fetching each via `get()`.
/// If `check_new` is true, also fetches the first page from `list()` and sends any new jobs
/// (not in `known_ids`) as `Action::JobsNewPrepended`.
pub fn refresh_visible_jobs(
    client: AzureClient,
    job_ids: Vec<String>,
    check_new: bool,
    known_ids: HashSet<String>,
    action_tx: ActionSender,
) {
    tokio::spawn(async move {
        let jobs_client = client.jobs();
        let rg = client.resource_group();
        let ws = client.workspace_name();

        // Fetch each visible job individually and collect updates
        let mut updated: Vec<JobRow> = Vec::new();
        for id in &job_ids {
            match jobs_client.get(rg, ws, id, None).await {
                Ok(response) => match response.into_model() {
                    Ok(job_base) => {
                        if let Some(row) = map_job_base(job_base) {
                            updated.push(row);
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Failed to deserialize job {}: {}", id, e);
                    }
                },
                Err(e) => {
                    // Job may have been deleted; skip it
                    tracing::debug!("Failed to fetch job {}: {}", id, e);
                }
            }
        }

        if !updated.is_empty() {
            let _ = action_tx.send(Action::JobsUpdated(updated));
        }

        // Check for new jobs if the top of the list is visible
        if check_new {
            match jobs_client.list(rg, ws, None) {
                Ok(mut pager) => {
                    let mut new_jobs: Vec<JobRow> = Vec::new();
                    // Read up to ~50 items from the first page
                    let mut count = 0;
                    while count < BATCH_SIZE {
                        match pager.next().await {
                            Some(Ok(job_base)) => {
                                if let Some(row) = map_job_base(job_base) {
                                    if !known_ids.contains(&row.id) {
                                        new_jobs.push(row);
                                    }
                                }
                                count += 1;
                            }
                            Some(Err(e)) => {
                                tracing::debug!("Error checking for new jobs: {}", e);
                                break;
                            }
                            None => break,
                        }
                    }
                    if !new_jobs.is_empty() {
                        let _ = action_tx.send(Action::JobsNewPrepended(new_jobs));
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to list jobs for new-job check: {}", e);
                }
            }
        }
    });
}
