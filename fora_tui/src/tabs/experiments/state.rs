use chrono::{DateTime, Utc};

use crate::tabs::recent_jobs::state::RecentJobRow;

/// Represents an experiment in the Experiments tab.
#[derive(Debug, Clone)]
pub struct ExperimentEntry {
    pub experiment_id: String,
    pub name: String,
    /// The start_time of the most recent job in this experiment.
    pub most_recent_job_time: Option<DateTime<Utc>>,
    /// Whether this experiment is expanded in the UI.
    pub expanded: bool,
    /// Jobs within this experiment (populated when expanded).
    pub jobs: Vec<RecentJobRow>,
    /// Whether jobs are currently being loaded for this experiment.
    pub loading_jobs: bool,
}

/// A row in the flat list — either an experiment header or a job under an experiment.
#[derive(Debug, Clone)]
pub enum ExperimentListItem {
    Experiment(usize), // index into experiments vec
    Job(usize, usize), // (experiment index, job index)
}

/// Tracks the state of experiment discovery.
#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryState {
    Idle,
    Loading,
    Complete,
    Refreshing,
}
