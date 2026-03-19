use std::collections::HashMap;

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
    /// Job status string (e.g. "FINISHED", "FAILED", "KILLED", "RUNNING").
    pub status: String,
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
}

/// State for the Recent Jobs tab.
#[derive(Debug, Default)]
pub struct RecentJobsState {
    pub detail_open: bool,
}

/// Tracks the state of the MLflow fetcher.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchState {
    Idle,
    Loading,
    Complete,
    Error,
}
