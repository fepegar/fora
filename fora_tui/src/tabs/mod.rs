pub mod compute;
pub mod experiments;
pub mod recent_jobs;

use std::sync::Arc;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::app::Action;
use crate::client::AzureClient;

/// Sender for dispatching actions from tabs back to the app.
pub type ActionSender = mpsc::UnboundedSender<Action>;

use azure_ml::models::JobStatus;

/// Check whether a job status indicates the job can be cancelled.
pub fn is_job_cancelable(status: &JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Running
            | JobStatus::Queued
            | JobStatus::Starting
            | JobStatus::Preparing
            | JobStatus::NotStarted
            | JobStatus::Provisioning
    )
}

/// Check whether a job status indicates the job hasn't started running yet.
pub fn is_pre_running_status(status: &JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Queued
            | JobStatus::Starting
            | JobStatus::Preparing
            | JobStatus::NotStarted
            | JobStatus::Provisioning
    )
}

/// Check whether a job status is non-terminal (job may still change).
pub fn is_active_status(status: &JobStatus) -> bool {
    !matches!(
        status,
        JobStatus::Completed | JobStatus::Failed | JobStatus::Canceled
    )
}

/// Spawn an async task that cancels a job and sends the result back via actions.
pub fn spawn_cancel_job(
    client: Arc<AzureClient>,
    job_id: String,
    display_name: String,
    action_tx: ActionSender,
) {
    tokio::spawn(async move {
        match client.cancel_job(&job_id).await {
            Ok(()) => {
                let _ = action_tx.send(Action::RefreshRequested);
            }
            Err(e) => {
                let _ = action_tx.send(Action::Error(format!(
                    "Failed to cancel job '{}': {}",
                    display_name, e
                )));
            }
        }
    });
}

/// Trait that all tabs must implement for extensibility.
pub trait Tab {
    /// Display title for the tab bar.
    fn title(&self) -> &str;

    /// Handle a key event. Returns true if the event was consumed.
    fn handle_key(&mut self, key: KeyEvent, action_tx: &ActionSender) -> bool;

    /// Process an action (e.g. data loaded, error).
    fn update(&mut self, action: &Action);

    /// Render the tab content into the given area.
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Called on each tick for periodic work (e.g. checking cache staleness).
    fn tick(&mut self, action_tx: &ActionSender);

    /// Key hints for the help bar in the current state.
    fn key_hints(&self) -> Vec<(&'static str, &'static str)>;
}
