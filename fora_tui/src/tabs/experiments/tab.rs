use std::collections::HashMap;
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use ratatui::Frame;
use tokio_util::sync::CancellationToken;

use crate::app::Action;
use crate::client::AzureClient;
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::detail_pane::{self, DetailKeyResult, DetailPane};
use crate::components::job_detail::JobDetail;
use crate::components::spinner::Spinner;
use crate::tabs::recent_jobs::columns::default_columns as job_default_columns;
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::tabs::{is_job_cancelable, spawn_cancel_job, ActionSender, Tab};
use crate::theme::{self, Theme};
use crate::widgets::table::ColumnDef;

use super::fetch;
use super::state::{DiscoveryState, ExperimentEntry, ExperimentListItem};

pub struct ExperimentsTab {
    experiments: Vec<ExperimentEntry>,
    /// Flattened list of items for display (experiment headers + expanded job rows).
    flat_list: Vec<ExperimentListItem>,
    table_state: TableState,
    total_items: usize,
    detail_open: bool,
    detail_pane: DetailPane,
    client: Option<AzureClient>,
    discovery_state: DiscoveryState,
    spinner: Spinner,
    /// Column definitions for job rows (same as Recent Jobs tab).
    job_columns: Vec<ColumnDef<RecentJobRow>>,
    cancel_token: CancellationToken,
    /// Cached experiment id → name mapping, preserved across refreshes for incremental fetching.
    experiment_cache: HashMap<String, String>,
    confirm_dialog: ConfirmDialog,
    /// Job ID + display name pending cancellation (set when confirm dialog is shown).
    pending_cancel_job_id: Option<(String, String)>,
}

impl ExperimentsTab {
    pub fn new(client: Option<AzureClient>, experiment_cache: HashMap<String, String>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        // Use the same columns as the Recent Jobs tab, but skip the "experiment"
        // column since it's redundant (the experiment header is right above).
        let job_columns: Vec<ColumnDef<RecentJobRow>> = job_default_columns()
            .into_iter()
            .filter(|c| c.id != "experiment")
            .collect();

        Self {
            experiments: Vec::new(),
            flat_list: Vec::new(),
            table_state,
            total_items: 0,
            detail_open: false,
            detail_pane: DetailPane::new(),
            client,
            discovery_state: DiscoveryState::Idle,
            spinner: Spinner::new(),
            job_columns,
            cancel_token: CancellationToken::new(),
            experiment_cache,
            confirm_dialog: ConfirmDialog::default(),
            pending_cancel_job_id: None,
        }
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        for (ei, exp) in self.experiments.iter().enumerate() {
            self.flat_list.push(ExperimentListItem::Experiment(ei));
            if exp.expanded {
                for ji in 0..exp.jobs.len() {
                    self.flat_list.push(ExperimentListItem::Job(ei, ji));
                }
            }
        }
        self.total_items = self.flat_list.len();

        // Adjust selection
        if let Some(sel) = self.table_state.selected() {
            if sel >= self.total_items && self.total_items > 0 {
                self.table_state.select(Some(self.total_items - 1));
            } else if self.total_items == 0 {
                self.table_state.select(None);
            }
        } else if self.total_items > 0 {
            self.table_state.select(Some(0));
        }
    }

    fn selected_item(&self) -> Option<&ExperimentListItem> {
        self.table_state
            .selected()
            .and_then(|i| self.flat_list.get(i))
    }

    fn selected_job(&self) -> Option<&RecentJobRow> {
        match self.selected_item()? {
            ExperimentListItem::Job(ei, ji) => {
                self.experiments.get(*ei).and_then(|e| e.jobs.get(*ji))
            }
            _ => None,
        }
    }

    fn toggle_expand(&mut self, action_tx: &ActionSender) {
        let Some(item) = self.selected_item().cloned() else {
            return;
        };

        match item {
            ExperimentListItem::Experiment(ei) => {
                let exp = &mut self.experiments[ei];
                if exp.expanded {
                    exp.expanded = false;
                    self.rebuild_flat_list();
                } else {
                    exp.expanded = true;
                    if exp.jobs.is_empty() && !exp.loading_jobs {
                        // Fetch jobs for this experiment
                        exp.loading_jobs = true;
                        if let Some(client) = self.client.clone() {
                            fetch::spawn_experiment_jobs_fetch(
                                client,
                                exp.experiment_id.clone(),
                                exp.name.clone(),
                                action_tx.clone(),
                                self.cancel_token.clone(),
                            );
                        }
                    }
                    self.rebuild_flat_list();
                }
            }
            ExperimentListItem::Job(_, _) => {
                self.detail_open = !self.detail_open;
                self.detail_pane.reset();
            }
        }
    }

    fn start_discovery(&mut self, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            return;
        };
        // Cancel any in-flight discovery and job fetch sub-tasks
        self.cancel_token.cancel();
        self.cancel_token = CancellationToken::new();

        self.discovery_state = DiscoveryState::Loading;
        self.experiments.clear();
        self.rebuild_flat_list();

        fetch::spawn_experiment_discovery(
            client,
            action_tx.clone(),
            self.cancel_token.clone(),
            self.experiment_cache.clone(),
        );
    }

    fn select_next(&mut self) {
        if self.total_items == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map(|i| (i + 1).min(self.total_items - 1))
            .unwrap_or(0);
        self.table_state.select(Some(i));
    }

    fn select_prev(&mut self) {
        if self.total_items == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.table_state.select(Some(i));
    }
}

impl Tab for ExperimentsTab {
    fn title(&self) -> &str {
        "Experiments"
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

        // When detail pane is open, delegate to DetailPane
        if self.detail_open {
            if let Some(job) = self.selected_job() {
                let run_id = job.id.clone();
                let metric_keys = job.metric_keys.clone();
                let result = self.detail_pane.handle_key(key, &run_id, &metric_keys);
                match result {
                    DetailKeyResult::Consumed => true,
                    DetailKeyResult::Close => {
                        self.detail_open = false;
                        self.detail_pane.reset();
                        true
                    }
                    DetailKeyResult::FetchMetrics {
                        run_id,
                        metric_keys,
                    } => {
                        if let Some(client) = self.client.clone() {
                            detail_pane::spawn_metrics_fetcher(
                                client,
                                run_id,
                                metric_keys,
                                action_tx.clone(),
                            );
                        }
                        true
                    }
                    DetailKeyResult::Ignored => false,
                }
            } else {
                self.detail_open = false;
                self.detail_pane.reset();
                true
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_prev();
                    true
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next();
                    true
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.toggle_expand(action_tx);
                    true
                }
                KeyCode::Esc => false,
                KeyCode::Char('r') => {
                    self.start_discovery(action_tx);
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
    }

    fn update(&mut self, action: &Action) {
        match action {
            Action::ExperimentsDiscovered(new_exps) => {
                for (exp_id, exp_name, most_recent_time) in new_exps {
                    // Only add if not already present
                    if !self.experiments.iter().any(|e| e.experiment_id == *exp_id) {
                        self.experiments.push(ExperimentEntry {
                            experiment_id: exp_id.clone(),
                            name: exp_name.clone(),
                            most_recent_job_time: *most_recent_time,
                            expanded: false,
                            jobs: Vec::new(),
                            loading_jobs: false,
                        });
                    }
                }
                self.rebuild_flat_list();
            }
            Action::ExperimentDiscoveryComplete => {
                self.discovery_state = DiscoveryState::Complete;
            }
            Action::ExperimentJobsLoaded {
                experiment_id,
                jobs,
            } => {
                if let Some(exp) = self
                    .experiments
                    .iter_mut()
                    .find(|e| e.experiment_id == *experiment_id)
                {
                    exp.jobs = jobs.clone();
                    exp.loading_jobs = false;
                }
                self.rebuild_flat_list();
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
                // Update enriched fields on jobs within experiments
                for exp in &mut self.experiments {
                    for job in &mut exp.jobs {
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
            }
            Action::ExperimentCacheUpdated(cache) => {
                self.experiment_cache = cache.clone();
            }
            Action::MetricsLoaded { .. } | Action::MetricsFetchFailed { .. } => {
                self.detail_pane.handle_action(action);
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.detail_open {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

            self.render_list(frame, chunks[0]);

            // Extract job data to avoid borrow conflict with detail_pane
            let job_data = self.selected_job().map(|job| {
                let created = job
                    .start_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string());
                let runtime = crate::format::format_runtime(job.start_time, job.end_time);
                (
                    job.id.clone(),
                    job.display_name.clone(),
                    job.experiment_name.clone(),
                    job.job_type.clone(),
                    job.status.clone(),
                    job.compute_target.clone(),
                    created,
                    runtime,
                    job.command.clone(),
                    job.environment_id.clone(),
                    job.description.clone(),
                    job.tags.clone(),
                )
            });

            if let Some((
                id,
                display_name,
                experiment_name,
                job_type,
                status,
                compute_target,
                created,
                runtime,
                command,
                environment_id,
                description,
                tags,
            )) = job_data
            {
                let detail = JobDetail {
                    id: &id,
                    display_name: &display_name,
                    experiment_name: &experiment_name,
                    job_type: job_type.as_deref().unwrap_or("—"),
                    status: &status,
                    compute_target: compute_target.as_deref().unwrap_or("—"),
                    created_at: created.as_deref(),
                    runtime: Some(runtime.as_str()),
                    command: command.as_deref(),
                    environment_id: environment_id.as_deref(),
                    description: description.as_deref(),
                    tags: if tags.is_empty() { None } else { Some(&tags) },
                };
                self.detail_pane.render(frame, chunks[1], &detail, &id);
            }
        } else {
            self.render_list(frame, area);
        }

        // Overlays
        self.confirm_dialog.render(frame, area);
    }

    fn tick(&mut self, action_tx: &ActionSender) {
        self.spinner.tick();

        if self.client.is_none() {
            return;
        }

        if self.discovery_state == DiscoveryState::Idle {
            self.start_discovery(action_tx);
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        if self.confirm_dialog.active {
            return vec![("y", "Yes"), ("n/Esc", "No"), ("←→", "Toggle")];
        }

        if self.detail_open {
            return vec![("←→", "Tab"), ("↑↓", "Scroll"), ("Esc", "Close Detail")];
        }

        let mut hints = vec![
            ("↑↓", "Navigate"),
            ("Enter", "Expand/Details"),
            ("r", "Refresh"),
        ];
        if self
            .selected_job()
            .is_some_and(|j| is_job_cancelable(&j.status))
        {
            hints.push(("x", "Cancel Job"));
        }
        hints
    }
}

impl ExperimentsTab {
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let status_indicator = match self.discovery_state {
            DiscoveryState::Idle | DiscoveryState::Loading => {
                format!(" {}", self.spinner.frame())
            }
            DiscoveryState::Complete => String::new(),
        };

        let block = Block::default()
            .title(format!(
                " Experiments [{}]{} ",
                self.experiments.len(),
                status_indicator
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style(true));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.flat_list.is_empty() {
            return;
        }

        // Determine visible job columns for the available width
        let visible_cols: Vec<&ColumnDef<RecentJobRow>> =
            self.job_columns.iter().filter(|c| c.visible).collect();

        // Build column widths: first column gets extra space for experiment headers
        let widths: Vec<Constraint> = visible_cols
            .iter()
            .map(|col| Constraint::Min(col.min_width.max(col.default_width)))
            .collect();

        // Header row using the job column labels
        let header_cells: Vec<Span> = visible_cols
            .iter()
            .map(|col| Span::styled(col.label, theme::header_style()))
            .collect();
        let header = Row::new(header_cells).height(1);

        // Build rows from flat list
        let rows: Vec<Row> = self
            .flat_list
            .iter()
            .enumerate()
            .map(|(idx, item)| match item {
                ExperimentListItem::Experiment(ei) => {
                    let exp = &self.experiments[*ei];
                    let arrow = if exp.expanded { "▾" } else { "▸" };
                    let loading = if exp.loading_jobs {
                        format!(" {}", self.spinner.frame())
                    } else {
                        String::new()
                    };
                    let job_count = if exp.expanded && !exp.jobs.is_empty() {
                        format!(" ({})", exp.jobs.len())
                    } else {
                        String::new()
                    };
                    let recent = exp
                        .most_recent_job_time
                        .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_default();

                    // Experiment header: bold name in first cell, last activity in last cell
                    let mut cells: Vec<Span> = Vec::with_capacity(visible_cols.len());
                    for (ci, _col) in visible_cols.iter().enumerate() {
                        if ci == 0 {
                            cells.push(Span::styled(
                                format!("{} {}{}{}", arrow, exp.name, job_count, loading),
                                Style::default()
                                    .fg(Theme::ACCENT)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        } else if ci == visible_cols.len() - 1 {
                            cells.push(Span::styled(
                                recent.clone(),
                                Style::default().fg(Theme::DIM),
                            ));
                        } else {
                            cells.push(Span::raw(""));
                        }
                    }
                    Row::new(cells).style(theme::stripe_style(idx))
                }
                ExperimentListItem::Job(ei, ji) => {
                    let job = &self.experiments[*ei].jobs[*ji];
                    // Use the same column extractors as the Recent Jobs tab
                    let cells: Vec<Span> = visible_cols
                        .iter()
                        .enumerate()
                        .map(|(ci, col)| {
                            let mut text = (col.extract)(job);
                            // Indent the first column to show it's nested
                            if ci == 0 {
                                text = format!("  {}", text);
                            }
                            let style = col
                                .style
                                .map(|f| f(job))
                                .unwrap_or_else(|| theme::stripe_style(idx));
                            Span::styled(text, style)
                        })
                        .collect();
                    Row::new(cells).style(theme::stripe_style(idx))
                }
            })
            .collect();

        let table = Table::new(rows, &widths)
            .header(header)
            .highlight_style(theme::selected_style())
            .highlight_symbol("▸ ");

        let mut ts = self.table_state.clone();
        frame.render_stateful_widget(table, inner, &mut ts);
    }
}
