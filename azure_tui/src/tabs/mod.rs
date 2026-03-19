pub mod compute;
pub mod experiments;
pub mod jobs;
pub mod recent_jobs;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::app::Action;

/// Sender for dispatching actions from tabs back to the app.
pub type ActionSender = mpsc::UnboundedSender<Action>;

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
