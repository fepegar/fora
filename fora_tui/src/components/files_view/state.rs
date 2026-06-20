//! State for the Files sub-tab.
//!
//! Holds the lazily-loaded directory tree, currently-rendered tree rows
//! (rebuilt on mutation), preview state, and a per-view
//! [`tokio_util::sync::CancellationToken`] held by a `DropGuard` so that
//! any of (a) closing the detail pane, (b) selecting a different job,
//! (c) switching workspaces (which drops the parent tab and hence this
//! state) immediately cancels all spawned list / content / tail tasks.

use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;

use bytes::Bytes;
use lru::LruCache;
use mlflow::ArtifactEntry;
use ratatui::widgets::ListState;
use tokio_util::sync::{CancellationToken, DropGuard};

use super::preview::PreviewKind;

/// LRU cap for cached previews. 16 files keeps memory bounded while still
/// being plenty for browsing a typical run.
const PREVIEW_CACHE_CAP: usize = 16;

/// Upper bound on the bytes retained for a single previewed file. Matches
/// the preview download cap in [`super::fetcher`]; tail appends keep only
/// the most recent bytes once this is exceeded so a growing log can't
/// accumulate unbounded memory.
const PREVIEW_BYTES_CAP: usize = 10 * 1024 * 1024;

/// Logical (POSIX-style) artifact path. The root is the empty string.
pub type ArtifactPath = String;

/// Which half of the Files sub-tab currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneFocus {
    Tree,
    Preview,
}

/// State for one logical directory in the artifact tree.
#[derive(Debug, Clone)]
pub enum DirState {
    NotLoaded,
    Loading,
    Loaded(Vec<ArtifactEntry>),
    Error(String),
}

/// State for the preview of one file.
#[derive(Debug, Clone)]
pub enum PreviewState {
    Loading,
    Loaded(CachedPreview),
    Error(String),
}

/// Bytes + validators + decoded form of a single previewed file.
#[derive(Debug, Clone)]
pub struct CachedPreview {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub bytes: Bytes,
    pub kind: PreviewKind,
}

/// One row in the flattened, rendered tree view.
#[derive(Debug, Clone)]
pub struct TreeRow {
    pub path: ArtifactPath,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    /// Only meaningful for files. `None` when the server reported `-1`.
    pub size: Option<u64>,
}

pub struct FilesState {
    pub current_run_id: Option<String>,
    pub focus: PaneFocus,
    pub dirs: HashMap<ArtifactPath, DirState>,
    pub open_dirs: HashSet<ArtifactPath>,
    pub visible_rows: Vec<TreeRow>,
    pub selected_idx: usize,
    /// Persistent list state for the tree widget so ratatui can preserve
    /// scroll offset across frames (mirrors ColumnPicker / WorkspacePicker).
    pub tree_list_state: ListState,
    pub tree_height: u16,
    pub preview_height: u16,
    pub preview_scroll: u16,
    /// True while the user is "following the tail" of the current preview
    /// (a log file). When true, [`Self::apply_content`] keeps scroll
    /// pinned to the bottom so live-tailing logs auto-advance. Cleared
    /// when the user scrolls up; restored when they scroll back to the
    /// bottom. Reset on every selection change.
    pub auto_tail_to_bottom: bool,
    pub previews: LruCache<ArtifactPath, PreviewState>,
    pub current_preview: Option<ArtifactPath>,
    /// Name of the syntect theme used when classifying highlighted text.
    pub syntax_theme: String,
    /// Lazy-built terminal graphics picker for image rendering. We try
    /// `Picker::from_query_stdio()` first; on failure (non-TTY, restricted
    /// container) we fall back to `Picker::halfblocks()` so something
    /// still renders.
    pub picker: Option<ratatui_image::picker::Picker>,

    /// Cancel-all sentinel for spawned tasks. Dropping this guard (e.g.
    /// when [`Self::reset`] is called or when the parent tab is replaced)
    /// cancels every list/content/tail task tied to its child token.
    cancel_guard: Option<DropGuard>,
    cancel_root: CancellationToken,
    tail_token: Option<CancellationToken>,
    tail_path: Option<ArtifactPath>,
}

impl Default for FilesState {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesState {
    pub fn new() -> Self {
        let cancel_root = CancellationToken::new();
        let cancel_guard = Some(cancel_root.clone().drop_guard());
        Self {
            current_run_id: None,
            focus: PaneFocus::Tree,
            dirs: HashMap::new(),
            open_dirs: HashSet::new(),
            visible_rows: Vec::new(),
            selected_idx: 0,
            tree_list_state: ListState::default(),
            tree_height: 0,
            preview_height: 0,
            preview_scroll: 0,
            auto_tail_to_bottom: false,
            previews: LruCache::new(NonZeroUsize::new(PREVIEW_CACHE_CAP).unwrap()),
            current_preview: None,
            syntax_theme: "base16-ocean.dark".to_string(),
            picker: None,
            cancel_root,
            cancel_guard,
            tail_token: None,
            tail_path: None,
        }
    }

    /// Cancel all in-flight artifact tasks tied to this view.
    pub fn cancel_all(&mut self) {
        // Cancel via the root token itself (drop guard also cancels on
        // Drop, but we may not be dropping the state here).
        self.cancel_root.cancel();
        // Build a fresh root for any subsequent spawn.
        self.cancel_root = CancellationToken::new();
        self.cancel_guard = Some(self.cancel_root.clone().drop_guard());
        self.tail_token = None;
        self.tail_path = None;
    }

    /// Cancel just the active tail loop, leaving other tasks alone.
    pub fn cancel_tail(&mut self) {
        if let Some(tok) = self.tail_token.take() {
            tok.cancel();
        }
        self.tail_path = None;
    }

    /// Start tracking a new tail and return a child token for it.
    ///
    /// Callers must call [`Self::cancel_tail`] first if there's already an
    /// active tail they want to replace.
    pub fn start_tail(&mut self, path: ArtifactPath) -> CancellationToken {
        let token = self.cancel_root.child_token();
        self.tail_token = Some(token.clone());
        self.tail_path = Some(path);
        token
    }

    /// Path being tailed, if any. Useful for tests.
    #[allow(dead_code)]
    pub fn tail_path(&self) -> Option<&ArtifactPath> {
        self.tail_path.as_ref()
    }

    /// Wipe state and cancel all tasks. Called when the parent detail
    /// pane is closed or a different job is selected.
    pub fn reset(&mut self) {
        self.cancel_all();
        self.current_run_id = None;
        self.dirs.clear();
        self.open_dirs.clear();
        self.visible_rows.clear();
        self.selected_idx = 0;
        self.preview_scroll = 0;
        self.auto_tail_to_bottom = false;
        self.previews.clear();
        self.current_preview = None;
        self.focus = PaneFocus::Tree;
    }

    /// Reset transient state when binding to a (possibly new) run.
    pub fn set_run(&mut self, run_id: String) {
        if self.current_run_id.as_deref() != Some(&run_id) {
            self.cancel_all();
            self.dirs.clear();
            self.open_dirs.clear();
            self.visible_rows.clear();
            self.previews.clear();
            self.current_preview = None;
            self.selected_idx = 0;
            self.preview_scroll = 0;
            self.auto_tail_to_bottom = false;
            self.focus = PaneFocus::Tree;
        }
        self.current_run_id = Some(run_id);
    }

    /// Return a child cancellation token for a freshly-spawned task.
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_root.child_token()
    }

    /// True when the given directory needs (re)loading: either it has never
    /// been requested, or a previous request failed (`DirState::Error`).
    pub fn dir_is_unloaded(&self, path: &str) -> bool {
        matches!(self.dirs.get(path), None | Some(DirState::Error(_)))
    }

    pub fn set_dir_loading(&mut self, path: &str) {
        self.dirs.insert(path.to_string(), DirState::Loading);
    }

    pub fn set_dir_loaded(&mut self, path: &str, entries: Vec<ArtifactEntry>) {
        // Server returns full child paths; we sort dirs first, then by name.
        let mut entries = entries;
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => leaf_name(&a.path).cmp(leaf_name(&b.path)),
        });
        self.dirs
            .insert(path.to_string(), DirState::Loaded(entries));
        if path.is_empty() {
            // Open the root directory implicitly so we see its children.
            self.open_dirs.insert(String::new());
        }
        self.rebuild_visible_rows();
    }

    pub fn set_dir_error(&mut self, path: &str, error: String) {
        self.dirs.insert(path.to_string(), DirState::Error(error));
        self.rebuild_visible_rows();
    }

    pub fn invalidate_dir(&mut self, path: &str) {
        self.dirs.remove(path);
        self.rebuild_visible_rows();
    }

    pub fn is_dir_open(&self, path: &str) -> bool {
        self.open_dirs.contains(path)
    }

    pub fn open_dir(&mut self, path: &str) {
        self.open_dirs.insert(path.to_string());
        self.rebuild_visible_rows();
    }

    pub fn close_dir(&mut self, path: &str) {
        self.open_dirs.remove(path);
        self.rebuild_visible_rows();
    }

    pub fn selected_row(&self) -> Option<&TreeRow> {
        self.visible_rows.get(self.selected_idx)
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.visible_rows.is_empty() {
            self.selected_idx = 0;
            return;
        }
        let len = self.visible_rows.len() as i32;
        let new = (self.selected_idx as i32)
            .saturating_add(delta)
            .clamp(0, len - 1);
        self.selected_idx = new as usize;
    }

    pub fn scroll_preview(&mut self, delta: i32) {
        if delta == i32::MIN {
            self.preview_scroll = 0;
            self.auto_tail_to_bottom = false;
            return;
        }
        if delta == i32::MAX {
            self.preview_scroll = u16::MAX;
            // Jumping to the end is an explicit "I want to follow the
            // bottom" gesture for log files.
            if let Some(path) = self.current_preview.as_deref() {
                if super::preview::is_log_path(path) {
                    self.auto_tail_to_bottom = true;
                }
            }
            return;
        }
        let new = (self.preview_scroll as i32).saturating_add(delta).max(0);
        let next: u16 = new.try_into().unwrap_or(u16::MAX);
        // Any upward motion drops out of auto-follow; explicit downward
        // motion that goes off the end re-arms it for log files.
        if delta < 0 {
            self.auto_tail_to_bottom = false;
        }
        self.preview_scroll = next;
    }

    /// Backspace: close the directory containing the selection, or go
    /// up one level if the selection is at root depth.
    pub fn collapse_or_ascend(&mut self) {
        let Some(row) = self.selected_row().cloned() else {
            return;
        };
        if row.is_dir && self.is_dir_open(&row.path) {
            self.close_dir(&row.path);
            return;
        }
        // Otherwise: close the parent directory and move selection there.
        let parent = parent_path(&row.path);
        if parent != row.path && self.is_dir_open(&parent) {
            self.close_dir(&parent);
            // Select the parent row.
            if let Some(idx) = self.visible_rows.iter().position(|r| r.path == parent) {
                self.selected_idx = idx;
            }
        }
    }

    pub fn previewed_path(&self) -> Option<&ArtifactPath> {
        self.current_preview.as_ref()
    }

    pub fn previewed_bytes(&self) -> Option<Bytes> {
        let path = self.current_preview.as_ref()?;
        match self.previews.peek(path)? {
            PreviewState::Loaded(p) => Some(p.bytes.clone()),
            _ => None,
        }
    }

    pub fn preview_needs_fetch(&self, path: &str) -> bool {
        match self.previews.peek(path) {
            None | Some(PreviewState::Error(_)) => true,
            Some(_) => false,
        }
    }

    pub fn select_preview(&mut self, path: ArtifactPath) {
        if self.current_preview.as_deref() != Some(&path) {
            self.preview_scroll = 0;
            // Log files default to following the tail (so live-tailed
            // output stays visible at the bottom). Non-log files start
            // at the top with no auto-follow.
            self.auto_tail_to_bottom = super::preview::is_log_path(&path);
        }
        self.current_preview = Some(path);
    }

    pub fn set_preview_loading(&mut self, path: &str) {
        self.previews.put(path.to_string(), PreviewState::Loading);
    }

    pub fn set_preview_error(&mut self, path: &str, error: String) {
        self.previews
            .put(path.to_string(), PreviewState::Error(error));
    }

    pub fn touch_preview(&mut self, path: &str) {
        // 304 from the tail loop — currently a no-op state-wise, but
        // promote in LRU to mark as "still in use".
        let _ = self.previews.get(path);
    }

    pub fn invalidate_preview(&mut self, path: &str) {
        self.previews.pop(path);
    }

    pub fn apply_content(
        &mut self,
        path: &str,
        bytes: Bytes,
        etag: Option<String>,
        last_modified: Option<String>,
        is_suffix: bool,
    ) {
        let merged_bytes = if is_suffix {
            if let Some(PreviewState::Loaded(existing)) = self.previews.peek(path) {
                append_capped(&existing.bytes, &bytes, PREVIEW_BYTES_CAP)
            } else {
                cap_tail(bytes, PREVIEW_BYTES_CAP)
            }
        } else {
            cap_tail(bytes, PREVIEW_BYTES_CAP)
        };

        let kind = super::preview::classify_with_theme(path, &merged_bytes, &self.syntax_theme);
        let cached = CachedPreview {
            etag,
            last_modified,
            bytes: merged_bytes,
            kind,
        };
        self.previews
            .put(path.to_string(), PreviewState::Loaded(cached));

        // Pin scroll to the bottom for log files we're following. Render
        // clamps to the last fully-visible row, so u16::MAX is safe.
        if self.auto_tail_to_bottom && self.current_preview.as_deref() == Some(path) {
            self.preview_scroll = u16::MAX;
        }
    }

    /// Used by force-refresh: returns either the directory currently
    /// containing the selection or the root if nothing is selected.
    pub fn current_directory_for_refresh(&self) -> String {
        match self.selected_row() {
            Some(r) if r.is_dir => r.path.clone(),
            Some(r) => parent_path(&r.path),
            None => String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.visible_rows.is_empty()
    }

    /// Rebuild the flat list of visible rows from `dirs` + `open_dirs`.
    fn rebuild_visible_rows(&mut self) {
        let previously_selected = self.selected_row().map(|r| r.path.clone());
        self.visible_rows.clear();

        // Always start from root entries.
        self.walk_dir("", 0);

        // Restore selection by path; fall back to clamping.
        if let Some(prev_path) = previously_selected {
            if let Some(idx) = self.visible_rows.iter().position(|r| r.path == prev_path) {
                self.selected_idx = idx;
                return;
            }
        }
        if self.selected_idx >= self.visible_rows.len() {
            self.selected_idx = self.visible_rows.len().saturating_sub(1);
        }
    }

    fn walk_dir(&mut self, dir_path: &str, depth: usize) {
        // Borrow-checker dance: clone the entry list first.
        let entries = match self.dirs.get(dir_path) {
            Some(DirState::Loaded(e)) => e.clone(),
            _ => return,
        };

        for entry in entries {
            let name = leaf_name(&entry.path).to_string();
            self.visible_rows.push(TreeRow {
                path: entry.path.clone(),
                name,
                depth,
                is_dir: entry.is_dir,
                size: entry.file_size,
            });
            if entry.is_dir && self.open_dirs.contains(&entry.path) {
                self.walk_dir(&entry.path, depth + 1);
            }
        }
    }
}

/// Return at most the last `cap` bytes of `b` (a zero-copy slice).
fn cap_tail(b: Bytes, cap: usize) -> Bytes {
    if b.len() > cap {
        b.slice(b.len() - cap..)
    } else {
        b
    }
}

/// Append `suffix` to `existing`, keeping only the trailing `cap` bytes of
/// the result. Only the portion of `existing` that survives the cap is
/// copied, avoiding churn when the preview is already near the cap.
fn append_capped(existing: &Bytes, suffix: &Bytes, cap: usize) -> Bytes {
    if suffix.len() >= cap {
        // The suffix alone fills (or overflows) the cap; drop `existing`.
        return cap_tail(suffix.clone(), cap);
    }
    let keep_existing = (cap - suffix.len()).min(existing.len());
    let mut joined = Vec::with_capacity(keep_existing + suffix.len());
    joined.extend_from_slice(&existing[existing.len() - keep_existing..]);
    joined.extend_from_slice(suffix);
    Bytes::from(joined)
}

/// Get the leaf (file/dir) name component of a logical artifact path.
pub fn leaf_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Get the parent directory of a logical artifact path. Returns `""`
/// (the root) for top-level entries and for any path with no separator.
pub fn parent_path(path: &str) -> String {
    match path.rfind('/') {
        Some(idx) => path[..idx].to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, is_dir: bool) -> ArtifactEntry {
        ArtifactEntry {
            path: path.to_string(),
            is_dir,
            file_size: None,
        }
    }

    #[test]
    fn root_listing_populates_visible_rows() {
        let mut state = FilesState::new();
        state.set_run("run-1".to_string());
        state.set_dir_loaded(
            "",
            vec![
                entry("user_logs", true),
                entry("std_log.txt", false),
                entry("metrics.json", false),
            ],
        );
        let names: Vec<_> = state.visible_rows.iter().map(|r| r.name.clone()).collect();
        assert_eq!(names, vec!["user_logs", "metrics.json", "std_log.txt"]);
    }

    #[test]
    fn opening_a_directory_inserts_children() {
        let mut state = FilesState::new();
        state.set_run("run-1".to_string());
        state.set_dir_loaded("", vec![entry("user_logs", true)]);
        state.open_dir("user_logs");
        state.set_dir_loaded("user_logs", vec![entry("user_logs/std_log.txt", false)]);
        let depths: Vec<usize> = state.visible_rows.iter().map(|r| r.depth).collect();
        assert_eq!(depths, vec![0, 1]);
        assert!(state.visible_rows[1].path == "user_logs/std_log.txt");
    }

    #[test]
    fn parent_path_returns_root_for_top_level() {
        assert_eq!(parent_path("foo.txt"), "");
        assert_eq!(parent_path("a/b/c"), "a/b");
        assert_eq!(parent_path(""), "");
    }

    #[test]
    fn move_selection_clamps_to_visible_rows() {
        let mut state = FilesState::new();
        state.set_run("r".into());
        state.set_dir_loaded("", vec![entry("a", false), entry("b", false)]);
        state.move_selection(100);
        assert_eq!(state.selected_idx, 1);
        state.move_selection(-100);
        assert_eq!(state.selected_idx, 0);
    }

    #[test]
    fn cancel_all_replaces_token_so_new_tasks_arent_pre_cancelled() {
        let mut state = FilesState::new();
        let t1 = state.cancel_token();
        state.cancel_all();
        assert!(t1.is_cancelled(), "old token should be cancelled");
        let t2 = state.cancel_token();
        assert!(!t2.is_cancelled(), "freshly minted token shouldn't be");
    }

    #[test]
    fn switching_run_id_resets_caches_and_cancels_tasks() {
        let mut state = FilesState::new();
        state.set_run("r1".into());
        state.set_dir_loaded("", vec![entry("a", false)]);
        let t1 = state.cancel_token();
        state.set_run("r2".into());
        assert!(t1.is_cancelled());
        assert!(state.visible_rows.is_empty());
    }

    #[test]
    fn apply_content_suffix_appends_to_existing_bytes() {
        let mut state = FilesState::new();
        state.set_run("r".into());
        state.set_dir_loaded("", vec![entry("std_log.txt", false)]);
        state.select_preview("std_log.txt".into());
        state.apply_content("std_log.txt", Bytes::from("hello "), None, None, false);
        state.apply_content("std_log.txt", Bytes::from("world"), None, None, true);
        let bytes = state.previewed_bytes().unwrap();
        assert_eq!(&bytes[..], b"hello world");
    }
}
