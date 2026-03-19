use std::collections::HashSet;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::app::Action;
use crate::client::AzureClient;
use crate::components::column_picker::{ColumnEntry, ColumnPicker};
use crate::components::search_bar::SearchBar;
use crate::components::workspace_picker::ConfirmDialog;
use crate::tabs::{ActionSender, Tab};
use crate::theme;
use crate::widgets::table::{self, ListState};

use super::columns::default_columns;
use super::detail::render_job_detail;
use super::fetch::{self, JobFetcherHandle, INITIAL_LOAD, PREFETCH_LOAD, PREFETCH_THRESHOLD};
use super::state::{FetchState, JobRow, JobsState};

pub struct JobsTab {
    all_jobs: Vec<JobRow>,
    filtered_jobs: Vec<JobRow>,
    list_state: ListState,
    columns: Vec<table::ColumnDef<JobRow>>,
    state: JobsState,
    search: SearchBar,
    column_picker: ColumnPicker,
    confirm_cancel: ConfirmDialog,
    client: Option<AzureClient>,
    fetcher: Option<JobFetcherHandle>,
    fetch_state: FetchState,
    last_refresh: Option<Instant>,
    refresh_interval: Duration,
    /// Height of the table data area (rows visible on screen), recorded during render.
    visible_height: u16,
    /// Whether an auto-refresh (in-place update) is currently in flight.
    auto_refreshing: bool,
}

impl JobsTab {
    pub fn new(
        client: Option<AzureClient>,
        refresh_interval: u64,
        column_config: Option<&[String]>,
    ) -> Self {
        let mut columns = default_columns();
        table::apply_column_config(&mut columns, column_config);

        Self {
            all_jobs: Vec::new(),
            filtered_jobs: Vec::new(),
            list_state: ListState::new(),
            columns,
            state: JobsState::default(),
            search: SearchBar::default(),
            column_picker: ColumnPicker::new(),
            confirm_cancel: ConfirmDialog::default(),
            client,
            fetcher: None,
            fetch_state: FetchState::Idle,
            last_refresh: None,
            refresh_interval: Duration::from_secs(refresh_interval),
            visible_height: 0,
            auto_refreshing: false,
        }
    }

    pub fn set_client(&mut self, client: AzureClient) {
        self.client = Some(client);
        self.reset_data();
    }

    fn reset_data(&mut self) {
        self.all_jobs.clear();
        self.filtered_jobs.clear();
        self.list_state.set_total(0);
        self.fetcher = None;
        self.fetch_state = FetchState::Idle;
        self.last_refresh = None;
    }

    fn apply_filter(&mut self) {
        if self.search.query.is_empty() {
            self.filtered_jobs = self.all_jobs.clone();
        } else {
            let query = self.search.query.to_lowercase();
            self.filtered_jobs = self
                .all_jobs
                .iter()
                .filter(|j| j.display_name.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }
        self.list_state.set_total(self.filtered_jobs.len());
    }

    fn selected_job(&self) -> Option<&JobRow> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_jobs.get(i))
    }

    fn start_fetch(&mut self, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            return;
        };
        // Drop any existing fetcher (cancels its task)
        self.fetcher = None;

        self.fetch_state = FetchState::Loading;
        self.last_refresh = Some(Instant::now());
        let handle = fetch::spawn_job_fetcher(client, action_tx.clone(), INITIAL_LOAD);
        self.fetcher = Some(handle);
    }

    fn refresh(&mut self, action_tx: &ActionSender) {
        self.all_jobs.clear();
        self.filtered_jobs.clear();
        self.list_state.set_total(0);
        self.auto_refreshing = false;
        self.start_fetch(action_tx);
    }

    /// Perform an in-place refresh of visible jobs without disrupting the UI.
    /// If the top of the list is visible, also check for new jobs.
    fn auto_refresh_visible(&mut self, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            return;
        };
        if self.auto_refreshing {
            return;
        }
        self.auto_refreshing = true;
        self.last_refresh = Some(Instant::now());

        // Determine the visible range in filtered_jobs
        let offset = self.list_state.table_state.offset();
        let visible_end = (offset + self.visible_height as usize).min(self.filtered_jobs.len());

        // Collect IDs of visible jobs
        let mut ids_to_refresh: Vec<String> = self.filtered_jobs[offset..visible_end]
            .iter()
            .map(|j| j.id.clone())
            .collect();

        // If detail pane is open, also refresh the selected job (may be outside visible range)
        if self.state.detail_open {
            if let Some(job) = self.selected_job() {
                if !ids_to_refresh.contains(&job.id) {
                    ids_to_refresh.push(job.id.clone());
                }
            }
        }

        // Check for new jobs if the top of the list is visible
        let check_new = offset == 0;
        let known_ids: HashSet<String> = self.all_jobs.iter().map(|j| j.id.clone()).collect();

        let tx = action_tx.clone();
        fetch::refresh_visible_jobs(client, ids_to_refresh, check_new, known_ids, tx);
    }

    /// Check if we should prefetch more items based on scroll position.
    fn maybe_prefetch(&self) {
        if self.fetch_state != FetchState::Paused {
            return;
        }
        let Some(selected) = self.list_state.selected() else {
            return;
        };
        let total = self.all_jobs.len();
        if total > 0 && selected + PREFETCH_THRESHOLD >= total {
            if let Some(ref handle) = self.fetcher {
                handle.load_more(PREFETCH_LOAD);
            }
        }
    }

    fn trigger_cancel(&mut self, action_tx: &ActionSender) {
        let Some(job) = self.selected_job() else {
            return;
        };
        let Some(client) = self.client.clone() else {
            return;
        };

        let job_id = job.id.clone();
        let tx = action_tx.clone();

        tokio::spawn(async move {
            let jobs_client = client.jobs();
            match jobs_client.cancel(
                client.resource_group(),
                client.workspace_name(),
                &job_id,
                None,
            ) {
                Ok(_) => {
                    let _ = tx.send(Action::JobCancelled(job_id));
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(format!("Failed to cancel job: {}", e)));
                }
            }
        });
    }
}

impl Tab for JobsTab {
    fn title(&self) -> &str {
        "Jobs"
    }

    fn handle_key(&mut self, key: KeyEvent, action_tx: &ActionSender) -> bool {
        // Modal handling takes priority
        if self.confirm_cancel.active {
            if let Some(confirmed) = self.confirm_cancel.handle_key(key) {
                if confirmed {
                    self.trigger_cancel(action_tx);
                }
                return true;
            }
            return true;
        }

        if self.column_picker.active {
            let was_active = self.column_picker.active;
            self.column_picker.handle_key(key);

            // Apply changes live so the table reflects them immediately
            if self.column_picker.changed {
                let updated = self.column_picker.get_columns();
                for (i, (id, visible)) in updated.iter().enumerate() {
                    if let Some(col) = self.columns.iter_mut().find(|c| c.id == *id) {
                        col.visible = *visible;
                        col.order = i;
                    }
                }
                self.columns.sort_by_key(|c| c.order);
            }

            // When picker closes with save requested, persist to config file
            if was_active && !self.column_picker.active && self.column_picker.save_requested {
                let visible_ids: Vec<String> = self
                    .columns
                    .iter()
                    .filter(|c| c.visible)
                    .map(|c| c.id.to_string())
                    .collect();
                let _ = action_tx.send(Action::SaveColumnConfig {
                    tab: "jobs".to_string(),
                    columns: visible_ids,
                });
            }
            return true;
        }

        if self.search.active {
            let consumed = self.search.handle_key(key);
            if consumed {
                self.apply_filter();
            }
            return consumed;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.select_prev();
                self.maybe_prefetch();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.select_next();
                self.maybe_prefetch();
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.state.detail_open = !self.state.detail_open;
                true
            }
            KeyCode::Esc => {
                if self.state.detail_open {
                    self.state.detail_open = false;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('/') => {
                self.search.open();
                true
            }
            KeyCode::Char('c') => {
                let entries: Vec<ColumnEntry> = self
                    .columns
                    .iter()
                    .map(|c| ColumnEntry {
                        id: c.id,
                        label: c.label,
                        visible: c.visible,
                    })
                    .collect();
                self.column_picker.open(entries);
                true
            }
            KeyCode::Char('r') => {
                self.refresh(action_tx);
                true
            }
            KeyCode::Char('x') => {
                if let Some(job) = self.selected_job() {
                    self.confirm_cancel
                        .show(format!("Cancel job '{}'?", job.display_name));
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, action: &Action) {
        match action {
            Action::JobsBatchLoaded(rows) => {
                self.all_jobs.extend(rows.iter().cloned());
                self.fetch_state = FetchState::Loading;
                self.apply_filter();
            }
            Action::JobsFetchPaused => {
                self.fetch_state = FetchState::Paused;
            }
            Action::JobsFetchComplete => {
                self.fetch_state = FetchState::Complete;
            }
            Action::JobsUpdated(rows) => {
                // Update existing jobs in-place by matching on ID
                for updated in rows {
                    if let Some(job) = self.all_jobs.iter_mut().find(|j| j.id == updated.id) {
                        *job = updated.clone();
                    }
                    if let Some(job) = self.filtered_jobs.iter_mut().find(|j| j.id == updated.id) {
                        *job = updated.clone();
                    }
                }
                self.auto_refreshing = false;
            }
            Action::JobsNewPrepended(rows) => {
                if !rows.is_empty() {
                    let new_count = rows.len();
                    // Prepend new jobs at the top of all_jobs
                    let mut new_all = rows.clone();
                    new_all.append(&mut self.all_jobs);
                    self.all_jobs = new_all;

                    // Re-apply filter (this rebuilds filtered_jobs)
                    let old_selected = self.list_state.selected();
                    self.apply_filter();

                    // Shift selection down by the number of new items that made it through
                    // the filter, so the cursor stays on the same job
                    if let Some(sel) = old_selected {
                        // Count how many of the new jobs appear in filtered_jobs
                        let new_ids: HashSet<&str> = rows.iter().map(|j| j.id.as_str()).collect();
                        let new_in_filter = self
                            .filtered_jobs
                            .iter()
                            .take(new_count)
                            .filter(|j| new_ids.contains(j.id.as_str()))
                            .count();
                        let new_sel = sel + new_in_filter;
                        self.list_state.table_state.select(Some(
                            new_sel.min(self.filtered_jobs.len().saturating_sub(1)),
                        ));
                    }
                }
                self.auto_refreshing = false;
            }
            Action::JobCancelled(_) => {
                // Will be picked up on next auto-refresh
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.state.detail_open {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

            // Record visible height: area height minus borders (2) minus header row (1)
            self.visible_height = chunks[0].height.saturating_sub(3);

            self.render_list(frame, chunks[0]);

            if let Some(job) = self.selected_job() {
                render_job_detail(frame, chunks[1], job);
            }
        } else {
            self.visible_height = area.height.saturating_sub(3);
            self.render_list(frame, area);
        }

        // Overlays
        self.search.render(frame, area);
        self.column_picker.render(frame, area);
        self.confirm_cancel.render(frame, area);
    }

    fn tick(&mut self, action_tx: &ActionSender) {
        if self.client.is_none() {
            return;
        }

        // Initial fetch
        if self.fetch_state == FetchState::Idle {
            self.start_fetch(action_tx);
            return;
        }

        // Auto-refresh when stale: update visible jobs in-place
        if self.fetch_state == FetchState::Paused || self.fetch_state == FetchState::Complete {
            if let Some(last) = self.last_refresh {
                if last.elapsed() > self.refresh_interval {
                    self.auto_refresh_visible(action_tx);
                }
            }
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        if self.confirm_cancel.active {
            return vec![("y", "Yes"), ("n", "No"), ("Esc", "Cancel")];
        }
        if self.column_picker.active {
            return self.column_picker.key_hints();
        }
        if self.search.active {
            return vec![("Esc", "Close"), ("Type", "Search")];
        }

        let mut hints = vec![
            ("↑↓", "Navigate"),
            ("Enter", "Details"),
            ("/", "Search"),
            ("c", "Columns"),
            ("x", "Cancel Job"),
            ("r", "Refresh"),
        ];
        if self.state.detail_open {
            hints.push(("Esc", "Close Detail"));
        }
        hints
    }
}

impl JobsTab {
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let status_indicator = match self.fetch_state {
            FetchState::Idle | FetchState::Loading => " ⟳",
            FetchState::Paused => " …",
            FetchState::Complete => "",
        };

        let block = Block::default()
            .title(format!(
                " Jobs [{}]{} ",
                self.filtered_jobs.len(),
                status_indicator
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style(true));

        let mut table_state = self.list_state.table_state.clone();
        table::render_table(
            frame,
            area,
            &self.columns,
            &self.filtered_jobs,
            &mut table_state,
            block,
        );
    }
}
