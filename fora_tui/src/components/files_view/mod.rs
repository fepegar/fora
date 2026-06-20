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

pub mod download;
pub mod fetcher;
pub mod highlight;
pub mod log_highlight;
pub mod preview;
pub mod state;
pub mod tree;

use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

use crate::app::Action;
use crate::client::AzureClient;
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::input_prompt::{InputOutcome, InputPrompt};
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

/// The artifact selected for download when the destination prompt opens.
#[derive(Debug, Clone)]
struct DownloadSource {
    artifact_path: String,
    is_dir: bool,
    item_name: String,
}

/// Files sub-tab state + render surface.
pub struct FilesView {
    client: Option<AzureClient>,
    state: FilesState,
    config: FilesConfig,
    last_status_active: bool,
    /// Destination prompt shown when downloading the highlighted item.
    input_prompt: InputPrompt,
    /// Overwrite confirmation shown when the destination already exists.
    confirm: ConfirmDialog,
    /// The item being downloaded, captured when the prompt opens.
    download_source: Option<DownloadSource>,
    /// Resolved destination awaiting overwrite confirmation.
    pending_dest: Option<std::path::PathBuf>,
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
            input_prompt: InputPrompt::default(),
            confirm: ConfirmDialog::default(),
            download_source: None,
            pending_dest: None,
        }
    }

    /// Called when the parent detail pane is reset (closed or job
    /// changed). Cancels any in-flight work and clears local state.
    pub fn reset(&mut self) {
        self.state.reset();
        self.cancel_download_flow();
    }

    /// Close any open download prompt / confirmation and drop the pending
    /// download.
    fn cancel_download_flow(&mut self) {
        self.input_prompt.close();
        self.confirm.active = false;
        self.download_source = None;
        self.pending_dest = None;
    }

    /// True while a download prompt or overwrite confirmation is open, so
    /// the parent routes keys here instead of treating them as sub-tab /
    /// pane controls.
    pub fn modal_active(&self) -> bool {
        self.input_prompt.active || self.confirm.active
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
        // Download modals take priority: while either is open, every key
        // drives the prompt / confirmation rather than tree navigation.
        if self.input_prompt.active {
            match self.input_prompt.handle_key(key) {
                InputOutcome::Pending => {}
                InputOutcome::Submitted(path) => self.on_destination_submitted(path, action_tx),
                InputOutcome::Cancelled => self.download_source = None,
            }
            return true;
        }
        if self.confirm.active {
            if let Some(confirmed) = self.confirm.handle_key(key) {
                if confirmed {
                    self.start_pending_download(action_tx);
                } else {
                    self.pending_dest = None;
                    self.download_source = None;
                }
            }
            return true;
        }

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
                self.begin_download(action_tx);
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

    /// Open the destination prompt for the highlighted file or directory,
    /// pre-filled with `<save_dir>/<name>`.
    fn begin_download(&mut self, action_tx: &ActionSender) {
        let Some(row) = self.state.selected_row().cloned() else {
            let _ = action_tx.send(Action::Error("No item selected to download".to_string()));
            return;
        };
        if self.state.current_run_id.is_none() {
            return;
        }
        // Derive the on-disk name from the artifact path through the
        // sanitiser so a hostile leaf (`..`, embedded separators) can't
        // redirect the write outside the chosen destination.
        let item_name = match download::sanitised_leaf(&row.path) {
            Ok(name) => name,
            Err(e) => {
                let _ = action_tx.send(Action::Error(format!("Cannot download: {:#}", e)));
                return;
            }
        };
        let default_dest = download::default_destination(&self.config.save_dir, &item_name);
        let kind = if row.is_dir { "directory" } else { "file" };
        self.download_source = Some(DownloadSource {
            artifact_path: row.path.clone(),
            is_dir: row.is_dir,
            item_name,
        });
        self.input_prompt
            .open(format!("Download {} to", kind), default_dest);
    }

    /// The user accepted a destination path: resolve it, then either ask
    /// for overwrite confirmation or start the download immediately.
    fn on_destination_submitted(&mut self, path: String, action_tx: &ActionSender) {
        let Some(source) = self.download_source.clone() else {
            return;
        };
        if path.is_empty() {
            // A blank input on Enter is a user cancellation, not a failure.
            let _ = action_tx.send(Action::Notice("Download cancelled".to_string()));
            self.download_source = None;
            return;
        }
        let resolved = match download::resolve_path(&path) {
            Ok(p) => p,
            Err(e) => {
                let _ = action_tx.send(Action::Error(format!("Invalid path: {:#}", e)));
                self.download_source = None;
                return;
            }
        };
        let dest = download::final_destination(&resolved, source.is_dir, &source.item_name);
        if download::destination_exists(&dest) {
            // A directory download replaces the destination wholesale
            // (existing contents are deleted), so spell that out rather than
            // a generic "Overwrite?".
            let msg = if source.is_dir {
                format!(
                    "{} exists. Replace it? Existing contents will be deleted.",
                    dest.display()
                )
            } else {
                format!("{} exists. Overwrite?", dest.display())
            };
            self.confirm.show(msg);
            self.pending_dest = Some(dest);
        } else {
            self.spawn_download_task(source, dest, action_tx);
            self.download_source = None;
        }
    }

    /// Overwrite confirmed: start the download to the pending destination.
    fn start_pending_download(&mut self, action_tx: &ActionSender) {
        let (Some(source), Some(dest)) = (self.download_source.take(), self.pending_dest.take())
        else {
            return;
        };
        self.spawn_download_task(source, dest, action_tx);
    }

    fn spawn_download_task(
        &mut self,
        source: DownloadSource,
        dest: std::path::PathBuf,
        action_tx: &ActionSender,
    ) {
        let Some(client) = self.client.clone() else {
            let _ = action_tx.send(Action::Error(
                "No Azure client: workspace not configured".to_string(),
            ));
            return;
        };
        let Some(run_id) = self.state.current_run_id.clone() else {
            return;
        };
        let token = self.state.cancel_token();
        download::spawn_download(
            client,
            run_id,
            source.artifact_path,
            source.is_dir,
            source.item_name,
            dest,
            token,
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

        // Download modals overlay the whole sub-tab area.
        self.input_prompt.render(frame, area);
        self.confirm.render(frame, area);
    }

    /// Key hints contributed by the Files sub-tab.
    pub fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("←→", "Focus"),
            ("↑↓", "Navigate"),
            ("Enter", "Open"),
            ("z", "Fullscreen"),
            ("s", "Download"),
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

    fn entry(path: &str, is_dir: bool) -> mlflow::ArtifactEntry {
        mlflow::ArtifactEntry {
            path: path.to_string(),
            is_dir,
            file_size: None,
        }
    }

    fn view_with_file() -> FilesView {
        let mut v = fresh_view();
        v.state.set_dir_loaded("", vec![entry("model.pkl", false)]);
        v
    }

    fn view_with_dir() -> FilesView {
        let mut v = fresh_view();
        v.state.set_dir_loaded("", vec![entry("checkpoints", true)]);
        v
    }

    fn unique_tmp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("fora-fv-{}-{}-{}", std::process::id(), nanos, name));
        p
    }

    #[test]
    fn s_with_no_rows_emits_error() {
        let (tx, mut rx) = mpsc::unbounded_channel::<Action>();
        let mut view = fresh_view();
        let consumed = view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        assert!(consumed);
        assert!(!view.modal_active());
        match rx.try_recv() {
            Ok(Action::Error(msg)) => assert!(msg.contains("No item selected")),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn s_opens_prompt_prefilled_for_file() {
        let (tx, _rx) = mpsc::unbounded_channel::<Action>();
        let mut view = view_with_file();
        let consumed = view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        assert!(consumed);
        assert!(view.modal_active());
        assert!(view.input_prompt.active);
        assert_eq!(view.input_prompt.value(), "./model.pkl");
        let src = view.download_source.as_ref().unwrap();
        assert!(!src.is_dir);
        assert_eq!(src.item_name, "model.pkl");
    }

    #[test]
    fn s_opens_prompt_for_directory() {
        let (tx, _rx) = mpsc::unbounded_channel::<Action>();
        let mut view = view_with_dir();
        view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        assert!(view.input_prompt.active);
        assert_eq!(view.input_prompt.value(), "./checkpoints");
        assert!(view.download_source.as_ref().unwrap().is_dir);
    }

    #[test]
    fn esc_cancels_prompt_and_clears_source() {
        let (tx, _rx) = mpsc::unbounded_channel::<Action>();
        let mut view = view_with_file();
        view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        assert!(view.modal_active());
        let consumed = view.handle_key(key(KeyCode::Esc), "run-123", &tx);
        assert!(consumed);
        assert!(!view.modal_active());
        assert!(view.download_source.is_none());
    }

    #[test]
    fn submitting_existing_path_opens_overwrite_confirm() {
        let (tx, _rx) = mpsc::unbounded_channel::<Action>();
        let existing = unique_tmp_path("exists.pkl");
        std::fs::write(&existing, b"x").unwrap();

        let mut view = view_with_file();
        view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        // Replace the prompt contents with the existing path and submit.
        view.input_prompt
            .open("Download file to", existing.to_string_lossy().into_owned());
        let consumed = view.handle_key(key(KeyCode::Enter), "run-123", &tx);

        assert!(consumed);
        assert!(view.confirm.active, "overwrite confirm should be shown");
        assert!(!view.input_prompt.active);
        assert_eq!(view.pending_dest.as_deref(), Some(existing.as_path()));

        let _ = std::fs::remove_file(&existing);
    }

    #[test]
    fn overwrite_decline_clears_pending() {
        let (tx, _rx) = mpsc::unbounded_channel::<Action>();
        let existing = unique_tmp_path("decline.pkl");
        std::fs::write(&existing, b"x").unwrap();

        let mut view = view_with_file();
        view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        view.input_prompt
            .open("Download file to", existing.to_string_lossy().into_owned());
        view.handle_key(key(KeyCode::Enter), "run-123", &tx);
        assert!(view.confirm.active);

        view.handle_key(key(KeyCode::Char('n')), "run-123", &tx);
        assert!(!view.confirm.active);
        assert!(view.pending_dest.is_none());
        assert!(view.download_source.is_none());

        let _ = std::fs::remove_file(&existing);
    }

    #[test]
    fn submitting_new_path_skips_confirm_and_reports_no_client() {
        let (tx, mut rx) = mpsc::unbounded_channel::<Action>();
        let target = unique_tmp_path("new.pkl");
        assert!(!target.exists());

        let mut view = view_with_file();
        view.handle_key(key(KeyCode::Char('s')), "run-123", &tx);
        view.input_prompt
            .open("Download file to", target.to_string_lossy().into_owned());
        view.handle_key(key(KeyCode::Enter), "run-123", &tx);

        assert!(!view.confirm.active, "no overwrite confirm for a new path");
        assert!(!view.modal_active());
        assert!(view.download_source.is_none());
        // With no client configured the spawn reports an error.
        let mut saw_no_client = false;
        while let Ok(action) = rx.try_recv() {
            if let Action::Error(msg) = action {
                if msg.contains("No Azure client") {
                    saw_no_client = true;
                }
            }
        }
        assert!(saw_no_client, "expected a 'No Azure client' error");
    }
}
