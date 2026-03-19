use std::collections::HashMap;

use azure_ml::models::JobStatus;
use chrono::{DateTime, Utc};

/// Flattened representation of a job for display.
#[derive(Debug, Clone)]
pub struct JobRow {
    pub id: String,
    pub display_name: String,
    pub experiment_name: String,
    pub status: JobStatus,
    pub compute_target: String,
    pub created_at: Option<DateTime<Utc>>,
    pub job_type: String,
    pub command: Option<String>,
    pub environment_id: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub properties: HashMap<String, String>,
}

/// State for the Jobs tab.
#[derive(Debug)]
pub struct JobsState {
    pub detail_open: bool,
}

impl Default for JobsState {
    fn default() -> Self {
        Self { detail_open: false }
    }
}

/// Tracks the state of the streaming fetcher.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchState {
    /// No fetcher running yet.
    Idle,
    /// Fetcher is actively loading items.
    Loading,
    /// Fetcher has paused after loading a batch; more data is available.
    Paused,
    /// Pager exhausted — all data has been loaded.
    Complete,
}
