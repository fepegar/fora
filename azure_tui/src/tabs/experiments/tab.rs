use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use ratatui::Frame;

use crate::app::Action;
use crate::client::AzureClient;
use crate::components::job_detail::{self, JobDetail};
use crate::components::spinner::Spinner;
use crate::tabs::recent_jobs::columns::default_columns as job_default_columns;
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::tabs::{ActionSender, Tab};
use crate::theme::{self, Theme};
use crate::widgets::table::ColumnDef;

use super::fetch;
use super::state::{DiscoveryState, ExperimentEntry, ExperimentListItem, ExperimentsState};

pub struct ExperimentsTab {
    experiments: Vec<ExperimentEntry>,
    /// Flattened list of items for display (experiment headers + expanded job rows).
    flat_list: Vec<ExperimentListItem>,
    table_state: TableState,
    total_items: usize,
    state: ExperimentsState,
    client: Option<AzureClient>,
    discovery_state: DiscoveryState,
    spinner: Spinner,
    /// Column definitions for job rows (same as Recent Jobs tab).
    job_columns: Vec<ColumnDef<RecentJobRow>>,
}

impl ExperimentsTab {
    pub fn new(client: Option<AzureClient>) -> Self {
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
            state: ExperimentsState::default(),
            client,
            discovery_state: DiscoveryState::Idle,
            spinner: Spinner::new(),
            job_columns,
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
                            );
                        }
                    }
                    self.rebuild_flat_list();
                }
            }
            ExperimentListItem::Job(_, _) => {
                self.state.detail_open = !self.state.detail_open;
            }
        }
    }

    fn start_discovery(&mut self, action_tx: &ActionSender) {
        let Some(client) = self.client.clone() else {
            return;
        };
        self.discovery_state = DiscoveryState::Loading;
        self.experiments.clear();
        self.rebuild_flat_list();

        fetch::spawn_experiment_discovery(client, action_tx.clone());
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
            KeyCode::Esc => {
                if self.state.detail_open {
                    self.state.detail_open = false;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('r') => {
                self.start_discovery(action_tx);
                true
            }
            _ => false,
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
        let mut hints = vec![
            ("↑↓", "Navigate"),
            ("Enter", "Expand/Details"),
            ("r", "Refresh"),
        ];
        if self.state.detail_open {
            hints.push(("Esc", "Close Detail"));
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
