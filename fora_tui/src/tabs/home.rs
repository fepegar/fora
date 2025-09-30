use super::Tab;
use crate::app::AppEvent;
use crate::keys::help_text;
use crate::navigation::{NavigationContext, TabNavigator};
use crate::select_keys;
use crate::{
    azure::{AzureClient, Experiment, Job, JobStatus},
    cache::CacheManager,
};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum HomeEvent {
    DataRefreshed,
}

pub struct HomeTab {
    // State
    recent_jobs: Vec<Job>,
    recent_experiments: Vec<Experiment>,
    loading: bool,

    // UI state - tracks which panel is focused
    focused_panel: HomePanel,
    jobs_list_state: ratatui::widgets::ListState,
    experiments_list_state: ratatui::widgets::ListState,

    // Dependencies
    azure_client: AzureClient,
    cache: CacheManager,
    event_tx: UnboundedSender<AppEvent>,
    navigator: Option<TabNavigator>,
}

#[derive(Debug, Clone, PartialEq)]
enum HomePanel {
    RecentJobs,
    RecentExperiments,
    Overview, // For future stats/metrics panel
}

impl HomeTab {
    pub fn new(
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Self {
        let mut jobs_list_state = ratatui::widgets::ListState::default();
        jobs_list_state.select(Some(0)); // Start with first item selected

        Self {
            recent_jobs: Vec::new(),
            recent_experiments: Vec::new(),
            loading: false,
            focused_panel: HomePanel::RecentJobs,
            jobs_list_state,
            experiments_list_state: ratatui::widgets::ListState::default(),
            azure_client,
            cache,
            event_tx,
            navigator: None,
        }
    }

    pub async fn handle_event(&mut self, event: HomeEvent) {
        match event {
            HomeEvent::DataRefreshed => {
                self.load_dashboard_data().await;
            }
        }
    }

    async fn load_dashboard_data(&mut self) {
        self.loading = true;

        // Load recent jobs directly
        if let Ok(jobs) = self.azure_client.get_recent_jobs().await {
            self.recent_jobs = jobs.into_iter().take(10).collect();
        }

        // Load recent experiments directly
        if let Ok(experiments) = self.azure_client.get_experiments().await {
            self.recent_experiments = experiments.into_iter().take(10).collect();
        }

        self.loading = false;
    }

    fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            HomePanel::RecentJobs => HomePanel::RecentExperiments,
            HomePanel::RecentExperiments => HomePanel::RecentJobs,
            HomePanel::Overview => HomePanel::RecentJobs,
        };

        // Reset selections when switching panels
        match self.focused_panel {
            HomePanel::RecentJobs => {
                if !self.recent_jobs.is_empty() {
                    self.jobs_list_state.select(Some(0));
                }
            }
            HomePanel::RecentExperiments => {
                if !self.recent_experiments.is_empty() {
                    self.experiments_list_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn navigate_current_panel(&mut self) {
        match self.focused_panel {
            HomePanel::RecentJobs => {
                if !self.recent_jobs.is_empty() {
                    let i = match self.jobs_list_state.selected() {
                        Some(i) => (i + 1) % self.recent_jobs.len(),
                        None => 0,
                    };
                    self.jobs_list_state.select(Some(i));
                }
            }
            HomePanel::RecentExperiments => {
                if !self.recent_experiments.is_empty() {
                    let i = match self.experiments_list_state.selected() {
                        Some(i) => (i + 1) % self.recent_experiments.len(),
                        None => 0,
                    };
                    self.experiments_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn navigate_current_panel_up(&mut self) {
        match self.focused_panel {
            HomePanel::RecentJobs => {
                if !self.recent_jobs.is_empty() {
                    let i = match self.jobs_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.recent_jobs.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.jobs_list_state.select(Some(i));
                }
            }
            HomePanel::RecentExperiments => {
                if !self.recent_experiments.is_empty() {
                    let i = match self.experiments_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.recent_experiments.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.experiments_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn select_current_item(&mut self) {
        match self.focused_panel {
            HomePanel::RecentJobs => {
                if let Some(selected) = self.jobs_list_state.selected() {
                    if let Some(job) = self.recent_jobs.get(selected) {
                        if let Some(navigator) = &self.navigator {
                            navigator.to_jobs(Some(job.id.clone()));
                        }
                    }
                }
            }
            HomePanel::RecentExperiments => {
                if let Some(selected) = self.experiments_list_state.selected() {
                    if let Some(experiment) = self.recent_experiments.get(selected) {
                        if let Some(navigator) = &self.navigator {
                            navigator.to_experiments(Some(experiment.id.clone()));
                        }
                    }
                }
            }
            HomePanel::Overview => {}
        }
    }

    fn get_job_status_symbol(&self, status: &JobStatus) -> &'static str {
        match status {
            JobStatus::Completed => "✓",
            JobStatus::Failed => "✗",
            JobStatus::Running => "⟳",
            JobStatus::Queued => "⏳",
            JobStatus::Canceled => "⏹",
            JobStatus::NotStarted => "○",
        }
    }

    fn get_job_status_color(&self, status: &JobStatus) -> Color {
        match status {
            JobStatus::Completed => Color::Green,
            JobStatus::Failed => Color::Red,
            JobStatus::Running => Color::Yellow,
            JobStatus::Queued => Color::Blue,
            JobStatus::Canceled => Color::Gray,
            JobStatus::NotStarted => Color::White,
        }
    }
}

#[async_trait]
impl Tab for HomeTab {
    async fn initialize(&mut self) {
        self.load_dashboard_data().await;
    }

    async fn refresh(&mut self) {
        self.load_dashboard_data().await;
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.next_panel();
            }
            KeyCode::Up => {
                self.navigate_current_panel_up();
            }
            KeyCode::Down => {
                self.navigate_current_panel();
            }
            select_keys!() => {
                self.select_current_item();
            }
            KeyCode::Char('j') => {
                if let Some(navigator) = &self.navigator {
                    navigator.to_jobs(None);
                }
            }
            KeyCode::Char('e') => {
                if let Some(navigator) = &self.navigator {
                    navigator.to_experiments(None);
                }
            }
            KeyCode::Char('c') => {
                if let Some(navigator) = &self.navigator {
                    navigator.to_compute();
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect) {
        // Create dashboard layout - 2 columns for now, expandable to 3 for future panels
        let dashboard_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Recent Jobs
                Constraint::Percentage(50), // Recent Experiments
                                            // Future: add Constraint::Percentage(33) for Overview/Stats panel
            ])
            .split(area);

        // Render dashboard panels
        self.render_recent_jobs(f, dashboard_chunks[0]);
        self.render_recent_experiments(f, dashboard_chunks[1]);
    }

    fn title(&self) -> &str {
        "Home"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn on_navigation(&mut self, context: &NavigationContext) {
        // Handle navigation context if needed
        // For example, if we came back from another tab, we might want to refresh data
        if context.previous_tab.is_some() {
            self.load_dashboard_data().await;
        }
    }

    fn set_navigator(&mut self, navigator: TabNavigator) {
        self.navigator = Some(navigator);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl HomeTab {
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let welcome_text = vec![Line::from(vec![
            Span::styled(
                "Azure ML ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Dashboard",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        let header_border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::ROUNDED
        };

        let header = Paragraph::new(welcome_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(header_border_set)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .alignment(Alignment::Center);

        f.render_widget(header, area);
    }

    fn render_recent_jobs(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == HomePanel::RecentJobs;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        if self.recent_jobs.is_empty() {
            let empty_text = if self.loading {
                "⏳ Loading recent jobs..."
            } else {
                "No recent jobs found.\nPress 'j' to go to Jobs tab."
            };

            let jobs_border_set = symbols::border::Set {
                top_left: symbols::line::VERTICAL_RIGHT,
                top_right: symbols::line::HORIZONTAL_DOWN,
                bottom_right: symbols::line::HORIZONTAL_UP,
                ..symbols::border::ROUNDED
            };

            let paragraph = Paragraph::new(empty_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📋 Recent Jobs")
                        .border_set(jobs_border_set)
                        .border_style(border_style),
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .recent_jobs
            .iter()
            .map(|job| {
                let status_symbol = self.get_job_status_symbol(&job.status);
                let status_color = self.get_job_status_color(&job.status);

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            status_symbol,
                            Style::default()
                                .fg(status_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(&job.name, Style::default().fg(Color::White)),
                    ]),
                    Line::from(Span::styled(
                        format!("  {}", job.experiment_name),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )),
                ])
            })
            .collect();

        let jobs_border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::HORIZONTAL_DOWN,
            bottom_right: symbols::line::HORIZONTAL_UP,
            ..symbols::border::ROUNDED
        };

        let jobs_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Recent Jobs")
                    .border_set(jobs_border_set)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(jobs_list, area, &mut self.jobs_list_state.clone());

        // Add help text at bottom if focused
        if is_focused && area.height > 8 {
            let help_area = Rect {
                x: area.x + 1,
                y: area.y + area.height - 3,
                width: area.width.saturating_sub(2),
                height: 2,
            };

            let help = Paragraph::new(format!(
                "{}: View details | {}: Navigate | {}: Switch panel",
                help_text::SELECT_KEYS,
                help_text::NAV_KEYS,
                help_text::SWITCH_PANEL_KEYS
            ))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

            f.render_widget(help, help_area);
        }
    }

    fn render_recent_experiments(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == HomePanel::RecentExperiments;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        if self.recent_experiments.is_empty() {
            let empty_text = if self.loading {
                "⏳ Loading recent experiments..."
            } else {
                "No recent experiments found.\nPress 'e' to go to Experiments tab."
            };

            let experiments_border_set = symbols::border::Set {
                top_left: symbols::line::VERTICAL_RIGHT,
                top_right: symbols::line::VERTICAL_LEFT,
                ..symbols::border::ROUNDED
            };

            let paragraph = Paragraph::new(empty_text)
                .block(
                    Block::default()
                        .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                        .title("🧪 Recent Experiments")
                        .border_set(experiments_border_set)
                        .border_style(border_style),
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .recent_experiments
            .iter()
            .map(|experiment| {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("🧪", Style::default().fg(Color::Blue)),
                        Span::raw(" "),
                        Span::styled(&experiment.name, Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!("  {} jobs", experiment.job_count),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::raw(" | "),
                        Span::styled(
                            experiment.created_time.format("%m/%d").to_string(),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                ])
            })
            .collect();

        let experiments_border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::ROUNDED
        };

        let experiments_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                    .title("🧪 Recent Experiments")
                    .border_set(experiments_border_set)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(
            experiments_list,
            area,
            &mut self.experiments_list_state.clone(),
        );

        // Add help text at bottom if focused
        if is_focused && area.height > 8 {
            let help_area = Rect {
                x: area.x + 1,
                y: area.y + area.height - 3,
                width: area.width.saturating_sub(2),
                height: 2,
            };

            let help = Paragraph::new("j: Jobs | e: Experiments | c: Compute")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

            f.render_widget(help, help_area);
        }
    }
}
