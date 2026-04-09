use std::collections::HashMap;

use azure_ml::models::JobStatus;
use chrono::{DateTime, Utc};

/// Flattened representation of a job for display in the Recent Jobs tab.
/// Starts with lightweight MLflow data; enriched fields are filled in by background tasks.
#[derive(Debug, Clone)]
pub struct RecentJobRow {
    /// The MLflow run_id / Azure ML job name.
    pub id: String,
    /// Display name (mlflow.runName tag or run_name).
    pub display_name: String,
    /// Experiment name (looked up from experiment_id).
    pub experiment_name: String,
    /// Experiment ID from MLflow.
    pub experiment_id: String,
    /// Job status.
    pub status: JobStatus,
    /// Start time as UTC datetime.
    pub start_time: Option<DateTime<Utc>>,
    /// End time as UTC datetime.
    pub end_time: Option<DateTime<Utc>>,
    /// Username from mlflow.user tag.
    pub user: Option<String>,

    // ── Enriched fields (populated by Azure ML jobs.get()) ──
    pub compute_target: Option<String>,
    pub job_type: Option<String>,
    pub command: Option<String>,
    pub environment_id: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub enriched: bool,
    /// Metric key names from the MLflow run data (for fetching history).
    pub metric_keys: Vec<String>,
}

/// Tracks the state of the MLflow fetcher.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchState {
    Idle,
    Loading,
    Complete,
    Refreshing,
    Error,
}
