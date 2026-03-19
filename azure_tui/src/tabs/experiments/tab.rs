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
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::tabs::{ActionSender, Tab};
use crate::theme::{self, Theme};

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
}

impl ExperimentsTab {
    pub fn new(client: Option<AzureClient>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            experiments: Vec::new(),
            flat_list: Vec::new(),
            table_state,
            total_items: 0,
            state: ExperimentsState::default(),
            client,
            discovery_state: DiscoveryState::Idle,
            spinner: Spinner::new(),
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
                    let recent = exp
                        .most_recent_job_time
                        .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_default();
                    let job_count = if exp.expanded && !exp.jobs.is_empty() {
                        format!(" ({})", exp.jobs.len())
                    } else {
                        String::new()
                    };
                    Row::new(vec![
                        Span::styled(
                            format!("{} {}{}{}", arrow, exp.name, job_count, loading),
                            Style::default()
                                .fg(Theme::ACCENT)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(recent, Style::default().fg(Theme::DIM)),
                    ])
                    .style(theme::stripe_style(idx))
                }
                ExperimentListItem::Job(ei, ji) => {
                    let job = &self.experiments[*ei].jobs[*ji];
                    let sym = mlflow_status_symbol(&job.status);
                    let name = format!("  {} {} {}", sym, job.status, job.display_name);
                    let time = job
                        .start_time
                        .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_default();
                    Row::new(vec![
                        Span::styled(name, mlflow_status_style(&job.status)),
                        Span::styled(time, Style::default().fg(Theme::DIM)),
                    ])
                    .style(theme::stripe_style(idx))
                }
            })
            .collect();

        let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];

        let header = Row::new(vec![
            Span::styled("Name", theme::header_style()),
            Span::styled("Last Activity", theme::header_style()),
        ])
        .height(1);

        let table = Table::new(rows, &widths)
            .header(header)
            .highlight_style(theme::selected_style())
            .highlight_symbol("▸ ");

        let mut ts = self.table_state.clone();
        frame.render_stateful_widget(table, inner, &mut ts);
    }
}

fn mlflow_status_symbol(status: &str) -> &'static str {
    match status {
        "FINISHED" => "✓",
        "FAILED" => "✗",
        "RUNNING" => "●",
        "KILLED" => "✕",
        "SCHEDULED" | "STARTING" => "◯",
        _ => "?",
    }
}

fn mlflow_status_style(status: &str) -> Style {
    match status {
        "FINISHED" => Style::default().fg(Theme::SUCCESS),
        "FAILED" => Style::default().fg(Theme::ERROR),
        "RUNNING" => Style::default().fg(Theme::RUNNING),
        "KILLED" => Style::default().fg(Theme::DIM),
        "SCHEDULED" | "STARTING" => Style::default().fg(Theme::WARNING),
        _ => Style::default().fg(Theme::DIM),
    }
}
