//! Files sub-tab for browsing and previewing a run's artifacts.
//!
//! Lives inside [`crate::components::detail_pane::DetailPane`] alongside
//! the Info and Metrics sub-tabs. Lazily loads the run's artifact tree via
//! the MLflow `artifacts/list` endpoint and previews selected files using
//! syntect for syntax highlighting, ansi-to-tui for log files, and
//! ratatui-image for images.
//!
//! All long-running work (directory listing, content download, live tail)
//! is spawned through [`fetcher`] with a single per-`FilesView`
//! [`tokio_util::sync::CancellationToken`] held via a `DropGuard`, so
//! switching jobs / closing the pane / switching workspaces reliably
//! cancels outstanding work.

pub mod fetcher;
pub mod highlight;
pub mod log_highlight;
pub mod preview;
pub mod save;
pub mod state;
pub mod tree;

use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::{is_active_status, ActionSender};

pub use state::{FilesState, PaneFocus};

/// Tunables for the Files sub-tab. Sourced from `[ui]` in the user
/// config and passed in at construction time.
#[derive(Debug, Clone)]
pub struct FilesConfig {
    /// Polling interval for the live-tail loop. `None` disables tailing.
    pub tail_interval: Option<Duration>,
    /// syntect theme name used for highlighted previews.
    pub syntax_theme: String,
    /// Directory where the `s` hotkey writes saved files (supports `~`).
    pub save_dir: String,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            tail_interval: Some(Duration::from_secs(3)),
            syntax_theme: "base16-ocean.dark".to_string(),
            save_dir: "./".to_string(),
        }
    }
}

/// Files sub-tab state + render surface.
pub struct FilesView {
    client: Option<AzureClient>,
    state: FilesState,
    config: FilesConfig,
    last_status_active: bool,
}

impl FilesView {
    pub fn new(client: Option<AzureClient>) -> Self {
        Self::with_config(client, FilesConfig::default())
    }

    pub fn with_config(client: Option<AzureClient>, config: FilesConfig) -> Self {
        let mut state = FilesState::new();
        state.syntax_theme = config.syntax_theme.clone();
        Self {
            client,
            state,
            config,
            last_status_active: false,
        }
    }

    /// Called when the parent detail pane is reset (closed or job
    /// changed). Cancels any in-flight work and clears local state.
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Called when the user switches *to* the Files sub-tab. Triggers an
    /// initial root-directory fetch if we haven't loaded one yet for the
    /// given run.
    pub fn on_enter(&mut self, run_id: &str, action_tx: &ActionSender) {
        if self.state.current_run_id.as_deref() != Some(run_id) {
            self.state.set_run(run_id.to_string());
        }
        if self.state.dir_is_unloaded("") {
            self.fetch_dir(run_id, "", action_tx);
        }
    }

    /// Called when the user switches *away* from the Files sub-tab.
    /// Stops live-tail polling while keeping caches intact.
    pub fn on_leave(&mut self) {
        self.state.cancel_tail();
    }

    /// Called from the parent on every render with the current run's
    /// status, so we can stop tailing when the run reaches a terminal
    /// state. Cheap when status is unchanged.
    pub fn on_status(&mut self, status: &azure_ml::models::JobStatus) {
        let active = is_active_status(status);
        if !active {
            self.state.cancel_tail();
        }
        self.last_status_active = active;
    }

    /// Process an artifact-related action.
    pub fn handle_action(&mut self, action: &Action) {
        match action {
            Action::RunArtifactListLoaded {
                run_id,
                path,
                entries,
            } => {
                if self.state.current_run_id.as_deref() == Some(run_id) {
                    self.state.set_dir_loaded(path, entries.clone());
                }
            }
            Action::RunArtifactListFailed {
                run_id,
                path,
                error,
            } => {
                if self.state.current_run_id.as_deref() == Some(run_id) {
                    self.state.set_dir_error(path, error.clone());
                }
            }
            Action::RunArtifactContentLoaded {
                run_id,
                path,
                bytes,
                etag,
                last_modified,
                is_suffix,
            } => {
                if self.state.current_run_id.as_deref() == Some(run_id) {
                    tracing::debug!(
                        run_id = %run_id,
                        path = %path,
                        len = bytes.len(),
                        is_suffix,
                        "FilesView: applying content",
                    );
                    self.state.apply_content(
                        path,
                        bytes.clone(),
                        etag.clone(),
                        last_modified.clone(),
                        *is_suffix,
                    );
                } else {
                    tracing::debug!(
                        action_run_id = %run_id,
                        state_run_id = ?self.state.current_run_id,
                        path = %path,
                        "FilesView: ignoring content action (run mismatch)",
                    );
                }
            }
            Action::RunArtifactContentFailed {
                run_id,
                path,
                error,
            } => {
                if self.state.current_run_id.as_deref() == Some(run_id) {
                    tracing::warn!(
                        run_id = %run_id,
                        path = %path,
                        error = %error,
                        "FilesView: content failed",
                    );
                    self.state.set_preview_error(path, error.clone());
                }
            }
            Action::RunArtifactNotModified { run_id, path } => {
                if self.state.current_run_id.as_deref() == Some(run_id) {
                    self.state.touch_preview(path);
                }
            }
            _ => {}
        }
    }

    /// Handle a key event. Returns `true` if consumed.
    pub fn handle_key(&mut self, key: KeyEvent, run_id: &str, action_tx: &ActionSender) -> bool {
        match key.code {
            KeyCode::Left => {
                self.state.focus = PaneFocus::Tree;
                true
            }
            KeyCode::Right => {
                self.state.focus = PaneFocus::Preview;
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.state.focus {
                    PaneFocus::Tree => self.state.move_selection(-1),
                    PaneFocus::Preview => self.state.scroll_preview(-1),
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.state.focus {
                    PaneFocus::Tree => self.state.move_selection(1),
                    PaneFocus::Preview => self.state.scroll_preview(1),
                }
                true
            }
            KeyCode::PageUp => {
                if matches!(self.state.focus, PaneFocus::Preview) {
                    let step = (self.state.preview_height as i32).max(1);
                    self.state.scroll_preview(-step);
                } else {
                    let step = (self.state.tree_height as i32).max(1);
                    self.state.move_selection(-step);
                }
                true
            }
            KeyCode::PageDown => {
                if matches!(self.state.focus, PaneFocus::Preview) {
                    let step = (self.state.preview_height as i32).max(1);
                    self.state.scroll_preview(step);
                } else {
                    let step = (self.state.tree_height as i32).max(1);
                    self.state.move_selection(step);
                }
                true
            }
            KeyCode::Home | KeyCode::Char('g') => {
                match self.state.focus {
                    PaneFocus::Tree => self.state.move_selection(i32::MIN),
                    PaneFocus::Preview => self.state.scroll_preview(i32::MIN),
                }
                true
            }
            KeyCode::End | KeyCode::Char('G') => {
                match self.state.focus {
                    PaneFocus::Tree => self.state.move_selection(i32::MAX),
                    PaneFocus::Preview => self.state.scroll_preview(i32::MAX),
                }
                true
            }
            KeyCode::Backspace => {
                self.state.collapse_or_ascend();
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_selected(run_id, action_tx);
                true
            }
            KeyCode::Char('s') => {
                self.save_selected(action_tx);
                true
            }
            KeyCode::Char('r') => {
                self.force_refresh(run_id, action_tx);
                true
            }
            KeyCode::Char('z') => {
                // Only meaningful when a file is actually being
                // previewed; on the bare tree the keypress is a no-op
                // (we still consume it so it doesn't bubble to global
                // handlers).
                if self.state.previewed_path().is_some() {
                    let _ = action_tx.send(Action::ToggleFullscreenPreview);
                }
                true
            }
            _ => false,
        }
    }

    fn activate_selected(&mut self, run_id: &str, action_tx: &ActionSender) {
        let Some(row) = self.state.selected_row().cloned() else {
            return;
        };
        if row.is_dir {
            if self.state.is_dir_open(&row.path) {
                self.state.close_dir(&row.path);
            } else {
                self.state.open_dir(&row.path);
                if self.state.dir_is_unloaded(&row.path) {
                    self.fetch_dir(run_id, &row.path, action_tx);
                }
            }
        } else {
            self.state.focus = PaneFocus::Preview;
            self.state.select_preview(row.path.clone());
            if self.state.preview_needs_fetch(&row.path) {
                self.fetch_content(run_id, &row.path, action_tx);
            }
            self.maybe_start_tail(run_id, &row.path, action_tx);
        }
    }

    fn save_selected(&mut self, action_tx: &ActionSender) {
        let Some(path) = self.state.previewed_path().cloned() else {
            let _ = action_tx.send(Action::Error("No file selected to save".to_string()));
            return;
        };
        let Some(bytes) = self.state.previewed_bytes() else {
            let _ = action_tx.send(Action::Error(
                "Preview not loaded yet — cannot save".to_string(),
            ));
            return;
        };
        let Some(run_id) = self.state.current_run_id.clone() else {
            return;
        };
        save::spawn_save(
            run_id,
            path,
            bytes,
            self.config.save_dir.clone(),
            action_tx.clone(),
        );
    }

    fn maybe_start_tail(&mut self, run_id: &str, path: &str, action_tx: &ActionSender) {
        let Some(interval) = self.config.tail_interval else {
            return;
        };
        if !self.last_status_active {
            return;
        }
        let Some(client) = self.client.clone() else {
            return;
        };
        // Cancel any in-flight tail (we may have switched files).
        self.state.cancel_tail();
        let tail_token = self.state.start_tail(path.to_string());
        let outer = self.state.cancel_token();
        fetcher::spawn_tail_loop(
            client,
            run_id.to_string(),
            path.to_string(),
            outer,
            tail_token,
            interval,
            action_tx.clone(),
        );
    }

    fn force_refresh(&mut self, run_id: &str, action_tx: &ActionSender) {
        let dir = self.state.current_directory_for_refresh();
        self.state.invalidate_dir(&dir);
        self.fetch_dir(run_id, &dir, action_tx);
        if let Some(path) = self.state.previewed_path().cloned() {
            self.state.invalidate_preview(&path);
            self.fetch_content(run_id, &path, action_tx);
            // Restart the tail so its internal etag/length resync with the
            // freshly re-fetched preview (maybe_start_tail cancels any
            // existing tail first).
            self.maybe_start_tail(run_id, &path, action_tx);
        }
    }

    fn fetch_dir(&mut self, run_id: &str, path: &str, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            self.state.set_dir_error(
                path,
                "No Azure client — workspace not configured".to_string(),
            );
            return;
        };
        self.state.set_dir_loading(path);
        let token = self.state.cancel_token();
        fetcher::spawn_list_fetcher(
            client,
            run_id.to_string(),
            path.to_string(),
            token,
            action_tx.clone(),
        );
    }

    fn fetch_content(&mut self, run_id: &str, path: &str, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            self.state.set_preview_error(
                path,
                "No Azure client — workspace not configured".to_string(),
            );
            return;
        };
        self.state.set_preview_loading(path);
        let token = self.state.cancel_token();
        fetcher::spawn_content_fetcher(
            client,
            run_id.to_string(),
            path.to_string(),
            token,
            action_tx.clone(),
        );
    }

    /// Render the Files sub-tab.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, _run_id: &str) {
        if area.width < 8 || area.height < 3 {
            return;
        }

        // Split: ~30% tree | ~70% preview (with a sensible minimum on each
        // and a cap on the tree so long file names don't dominate over
        // the preview).
        let tree_width = ((area.width * 3) / 10)
            .clamp(22, 36)
            .min(area.width.saturating_sub(24));
        let chunks =
            Layout::horizontal([Constraint::Length(tree_width), Constraint::Min(20)]).split(area);

        let tree_area = chunks[0];
        let preview_area = chunks[1];

        self.state.tree_height = tree_area.height;
        self.state.preview_height = preview_area.height;

        tree::render_tree(frame, tree_area, &mut self.state);
        preview::render_preview(frame, preview_area, &mut self.state);
    }

    /// Key hints contributed by the Files sub-tab.
    pub fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("←→", "Focus"),
            ("↑↓", "Navigate"),
            ("Enter", "Open"),
            ("z", "Fullscreen"),
            ("s", "Save"),
            ("r", "Refresh"),
        ]
    }

    /// True when there's a file currently selected for preview. Used by
    /// the app/detail-pane to decide whether `z` should enter fullscreen.
    pub fn has_preview(&self) -> bool {
        self.state.previewed_path().is_some()
    }

    /// Render only the preview pane into the given area. Used when the
    /// user requests a fullscreen preview, so we hide the tree pane and
    /// give the entire surface to the content. Updates
    /// `state.preview_height` so PgUp/PgDn page sizes match the
    /// fullscreen viewport.
    pub fn render_fullscreen(&mut self, frame: &mut Frame, area: Rect) {
        if area.width < 4 || area.height < 1 {
            return;
        }
        self.state.preview_height = area.height;
        // Force preview focus while fullscreen so PgUp/PgDn/g/G drive
        // the scroll directly even if the tree was focused before.
        self.state.focus = PaneFocus::Preview;
        preview::render_preview(frame, area, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    //! Regression tests for the key plumbing in `FilesView` that's hard
    //! to verify without a full TUI integration test.

    use super::*;
    use crate::app::Action;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tokio::sync::mpsc;

    fn fresh_view() -> FilesView {
        let mut v = FilesView::new(None);
        v.state.set_run("run-123".to_string());
        v
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn z_is_noop_when_no_file_selected() {
        let (tx, mut rx) = mpsc::unbounded_channel::<Action>();
        let mut view = fresh_view();
        let consumed = view.handle_key(key(KeyCode::Char('z')), "run-123", &tx);
        // Key is consumed (so it doesn't bubble to global handlers) but
        // no action is sent because there's nothing to fullscreen.
        assert!(consumed, "'z' should be consumed even without a preview");
        assert!(rx.try_recv().is_err(), "no action expected");
    }

    #[test]
    fn z_emits_toggle_action_when_file_is_selected() {
        let (tx, mut rx) = mpsc::unbounded_channel::<Action>();
        let mut view = fresh_view();
        view.state.select_preview("std_log.txt".to_string());
        let consumed = view.handle_key(key(KeyCode::Char('z')), "run-123", &tx);
        assert!(consumed);
        match rx.try_recv() {
            Ok(Action::ToggleFullscreenPreview) => {}
            other => panic!("expected ToggleFullscreenPreview, got {:?}", other),
        }
    }

    #[test]
    fn has_preview_reflects_selection() {
        let mut view = fresh_view();
        assert!(!view.has_preview());
        view.state.select_preview("std_log.txt".to_string());
        assert!(view.has_preview());
    }
}
