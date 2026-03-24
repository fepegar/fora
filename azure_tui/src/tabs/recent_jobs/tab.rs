use std::collections::HashMap;
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tokio_util::sync::CancellationToken;

use crate::app::Action;
use crate::client::AzureClient;
use crate::components::column_picker::{ColumnEntry, ColumnPicker};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::job_detail::{self, JobDetail};
use crate::components::search_bar::SearchBar;
use crate::components::spinner::Spinner;
use crate::tabs::{is_job_cancelable, spawn_cancel_job, ActionSender, Tab};
use crate::theme;
use crate::widgets::table::{self, ListState};

use super::columns::default_columns;
use super::fetch;
use super::state::{FetchState, RecentJobRow, RecentJobsState};

pub struct RecentJobsTab {
    all_jobs: Vec<RecentJobRow>,
    filtered_jobs: Vec<RecentJobRow>,
    list_state: ListState,
    columns: Vec<table::ColumnDef<RecentJobRow>>,
    state: RecentJobsState,
    search: SearchBar,
    column_picker: ColumnPicker,
    client: Option<AzureClient>,
    username: String,
    fetch_state: FetchState,
    spinner: Spinner,
    cancel_token: CancellationToken,
    /// Cached experiment id → name mapping, preserved across refreshes for incremental fetching.
    experiment_cache: HashMap<String, String>,
    confirm_dialog: ConfirmDialog,
    /// Job ID pending cancellation (set when confirm dialog is shown).
    pending_cancel_job_id: Option<(String, String)>,
}

impl RecentJobsTab {
    pub fn new(
        client: Option<AzureClient>,
        username: String,
        column_config: Option<&[String]>,
        experiment_cache: HashMap<String, String>,
    ) -> Self {
        let mut columns = default_columns();
        table::apply_column_config(&mut columns, column_config);

        Self {
            all_jobs: Vec::new(),
            filtered_jobs: Vec::new(),
            list_state: ListState::new(),
            columns,
            state: RecentJobsState::default(),
            search: SearchBar::default(),
            column_picker: ColumnPicker::new(),
            client,
            username,
            fetch_state: FetchState::Idle,
            spinner: Spinner::new(),
            cancel_token: CancellationToken::new(),
            experiment_cache,
            confirm_dialog: ConfirmDialog::default(),
            pending_cancel_job_id: None,
        }
    }

    fn apply_filter(&mut self) {
        if self.search.query.is_empty() {
            self.filtered_jobs = self.all_jobs.clone();
        } else {
            let query = self.search.query.to_lowercase();
            self.filtered_jobs = self
                .all_jobs
                .iter()
                .filter(|j| {
                    j.display_name.to_lowercase().contains(&query)
                        || j.experiment_name.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
        self.list_state.set_total(self.filtered_jobs.len());
    }

    fn selected_job(&self) -> Option<&RecentJobRow> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_jobs.get(i))
    }

    fn start_fetch(&mut self, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            return;
        };
        // Cancel any in-flight fetcher and its enrichment sub-tasks
        self.cancel_token.cancel();
        self.cancel_token = CancellationToken::new();

        self.fetch_state = FetchState::Loading;
        self.all_jobs.clear();
        self.filtered_jobs.clear();
        self.list_state.set_total(0);

        fetch::spawn_recent_jobs_fetcher(
            client,
            self.username.clone(),
            action_tx.clone(),
            self.cancel_token.clone(),
            self.experiment_cache.clone(),
        );
    }
}

impl Tab for RecentJobsTab {
    fn title(&self) -> &str {
        "Recent Jobs"
    }

    fn handle_key(&mut self, key: KeyEvent, action_tx: &ActionSender) -> bool {
        // Confirm dialog takes priority over everything
        if self.confirm_dialog.active {
            if let Some(confirmed) = self.confirm_dialog.handle_key(key) {
                if confirmed {
                    if let Some((job_id, display_name)) = self.pending_cancel_job_id.take() {
                        if let Some(client) = self.client.clone() {
                            spawn_cancel_job(
                                Arc::new(client),
                                job_id,
                                display_name,
                                action_tx.clone(),
                            );
                        }
                    }
                } else {
                    self.pending_cancel_job_id = None;
                }
            }
            return true;
        }

        if self.column_picker.active {
            let was_active = self.column_picker.active;
            self.column_picker.handle_key(key);

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

            if was_active && !self.column_picker.active && self.column_picker.save_requested {
                let visible_ids: Vec<String> = self
                    .columns
                    .iter()
                    .filter(|c| c.visible)
                    .map(|c| c.id.to_string())
                    .collect();
                let _ = action_tx.send(Action::SaveColumnConfig {
                    tab: "recent_jobs".to_string(),
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
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.select_next();
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
                self.start_fetch(action_tx);
                true
            }
            KeyCode::Char('x') => {
                if let Some(job) = self.selected_job() {
                    if is_job_cancelable(&job.status) {
                        let job_id = job.id.clone();
                        let display_name = job.display_name.clone();
                        self.confirm_dialog
                            .show(format!("Cancel job '{}'?", display_name));
                        self.pending_cancel_job_id = Some((job_id, display_name));
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, action: &Action) {
        match action {
            Action::RecentJobsBatchLoaded(rows) => {
                self.all_jobs.extend(rows.iter().cloned());
                self.apply_filter();
            }
            Action::RecentJobsFetchComplete => {
                self.fetch_state = FetchState::Complete;
            }
            Action::RecentJobEnriched {
                job_id,
                compute_target,
                job_type,
                command,
                environment_id,
                description,
                tags,
            } => {
                for job in self
                    .all_jobs
                    .iter_mut()
                    .chain(self.filtered_jobs.iter_mut())
                {
                    if job.id == *job_id {
                        job.compute_target = compute_target.clone();
                        job.job_type = job_type.clone();
                        job.command = command.clone();
                        job.environment_id = environment_id.clone();
                        job.description = description.clone();
                        job.tags = tags.clone();
                        job.enriched = true;
                    }
                }
            }
            Action::ExperimentCacheUpdated(cache) => {
                self.experiment_cache = cache.clone();
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.state.detail_open {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

            self.render_list(frame, chunks[0]);

            if let Some(job) = self.selected_job() {
                let created = job
                    .start_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string());
                let detail = JobDetail {
                    id: &job.id,
                    display_name: &job.display_name,
                    experiment_name: &job.experiment_name,
                    job_type: job.job_type.as_deref().unwrap_or("—"),
                    status: &job.status,
                    compute_target: job.compute_target.as_deref().unwrap_or("—"),
                    created_at: created.as_deref(),
                    command: job.command.as_deref(),
                    environment_id: job.environment_id.as_deref(),
                    description: job.description.as_deref(),
                    tags: if job.tags.is_empty() {
                        None
                    } else {
                        Some(&job.tags)
                    },
                };
                job_detail::render_job_detail(frame, chunks[1], &detail);
            }
        } else {
            self.render_list(frame, area);
        }

        // Overlays
        self.search.render(frame, area);
        self.column_picker.render(frame, area);
        self.confirm_dialog.render(frame, area);
    }

    fn tick(&mut self, action_tx: &ActionSender) {
        self.spinner.tick();

        if self.client.is_none() {
            return;
        }

        if self.fetch_state == FetchState::Idle {
            self.start_fetch(action_tx);
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        if self.confirm_dialog.active {
            return vec![("y", "Yes"), ("n/Esc", "No"), ("←→", "Toggle")];
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
            ("r", "Refresh"),
        ];
        if self
            .selected_job()
            .is_some_and(|j| is_job_cancelable(&j.status))
        {
            hints.push(("x", "Cancel Job"));
        }
        if self.state.detail_open {
            hints.push(("Esc", "Close Detail"));
        }
        hints
    }
}

impl RecentJobsTab {
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let status_indicator = match self.fetch_state {
            FetchState::Idle | FetchState::Loading => {
                format!(" {}", self.spinner.frame())
            }
            FetchState::Complete | FetchState::Error => String::new(),
        };

        let block = Block::default()
            .title(format!(
                " Recent Jobs [{}]{} ",
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
