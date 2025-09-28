use super::Tab;
use crate::app::AppEvent;
use crate::navigation::{NavigationContext, TabNavigator};
use crate::{
    azure::{AzureClient, Experiment, Job},
    cache::CacheManager,
};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum ExperimentsEvent {
    ExperimentSelected(String),
    ExperimentJobsLoaded(String, Vec<Job>),
    ShowExperimentJobs(bool),
}

pub struct ExperimentsTab {
    // State
    experiments: Vec<Experiment>,
    selected_experiment: Option<String>,
    experiment_jobs: Vec<Job>,
    show_jobs: bool,
    loading: bool,

    // UI state
    experiments_list_state: ListState,
    jobs_list_state: ListState,

    // Dependencies
    azure_client: AzureClient,
    cache: CacheManager,
    event_tx: UnboundedSender<AppEvent>,
    navigator: Option<TabNavigator>,
}

impl ExperimentsTab {
    pub fn new(
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Self {
        Self {
            experiments: Vec::new(),
            selected_experiment: None,
            experiment_jobs: Vec::new(),
            show_jobs: false,
            loading: false,
            experiments_list_state: ListState::default(),
            jobs_list_state: ListState::default(),
            azure_client,
            cache,
            event_tx,
            navigator: None,
        }
    }

    pub async fn handle_event(&mut self, event: ExperimentsEvent) {
        match event {
            ExperimentsEvent::ExperimentSelected(exp_id) => {
                self.selected_experiment = Some(exp_id.clone());
                self.show_jobs = true;
                self.load_experiment_jobs(exp_id).await;
            }
            ExperimentsEvent::ExperimentJobsLoaded(exp_id, jobs) => {
                if Some(&exp_id) == self.selected_experiment.as_ref() {
                    self.experiment_jobs = jobs;
                }
            }
            ExperimentsEvent::ShowExperimentJobs(show) => {
                self.show_jobs = show;
                if !show {
                    self.selected_experiment = None;
                    self.experiment_jobs.clear();
                }
            }
        }
    }

    async fn load_experiments(&mut self) {
        tracing::debug!("Loading experiments");

        // Always show cached data immediately if available (even if expired)
        let cached_experiments = self.cache.get_experiments_cached();
        if !cached_experiments.is_empty() && self.experiments.is_empty() {
            self.experiments = cached_experiments;
            tracing::debug!(
                "Loaded {} experiments from cache for immediate display",
                self.experiments.len()
            );
        }

        // Start loading fresh data in background
        self.loading = true;
        let client = self.azure_client.clone();
        let cache = self.cache.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            tracing::debug!("Fetching fresh experiments from Azure");
            match client.get_experiments().await {
                Ok(experiments) => {
                    tracing::debug!("Fetched {} experiments from Azure", experiments.len());
                    cache.store_experiments(experiments).await;
                    // Send refresh event to update UI with new data
                    let _ = tx.send(AppEvent::Refresh);
                }
                Err(e) => {
                    tracing::error!("Failed to load experiments: {}", e);
                }
            }
        });

        // If we had cached data, we're not really loading from user's perspective
        if !self.experiments.is_empty() {
            self.loading = false;
        }
    }

    async fn load_experiment_jobs(&mut self, experiment_id: String) {
        let client = self.azure_client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            match client.get_experiment_jobs(&experiment_id).await {
                Ok(jobs) => {
                    let _ = tx.send(AppEvent::ExperimentsEvent(
                        ExperimentsEvent::ExperimentJobsLoaded(experiment_id, jobs),
                    ));
                }
                Err(e) => {
                    tracing::error!("Failed to load experiment jobs: {}", e);
                }
            }
        });
    }
}

#[async_trait]
impl Tab for ExperimentsTab {
    async fn initialize(&mut self) {
        // Skip if we already have data loaded to avoid redundant work
        if !self.experiments.is_empty() {
            tracing::debug!("Experiments tab already has data, skipping initialization");
            return;
        }

        tracing::debug!("Initializing Experiments tab");

        // Load cached data first for immediate display (even if expired)
        let cached_experiments = self.cache.get_experiments_cached();
        if !cached_experiments.is_empty() {
            self.experiments = cached_experiments;
            tracing::debug!("Loaded {} experiments from cache", self.experiments.len());
        }

        // Then refresh in background
        self.load_experiments().await;
    }

    async fn refresh(&mut self) {
        // Always refresh from cache first for immediate update
        let cached_experiments = self.cache.get_experiments_cached();
        if !cached_experiments.is_empty() {
            self.experiments = cached_experiments;
        }

        // Stop loading state if we have fresh data
        self.loading = false;

        // Only reload if we don't have any data
        if self.experiments.is_empty() {
            self.load_experiments().await;
        }
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.show_jobs {
                    // Navigate jobs list
                    if !self.experiment_jobs.is_empty() {
                        let i = match self.jobs_list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.experiment_jobs.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.jobs_list_state.select(Some(i));
                    }
                } else {
                    // Navigate experiments list
                    if !self.experiments.is_empty() {
                        let i = match self.experiments_list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.experiments.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.experiments_list_state.select(Some(i));
                    }
                }
            }
            KeyCode::Down => {
                if self.show_jobs {
                    // Navigate jobs list
                    if !self.experiment_jobs.is_empty() {
                        let i = match self.jobs_list_state.selected() {
                            Some(i) => (i + 1) % self.experiment_jobs.len(),
                            None => 0,
                        };
                        self.jobs_list_state.select(Some(i));
                    }
                } else {
                    // Navigate experiments list
                    if !self.experiments.is_empty() {
                        let i = match self.experiments_list_state.selected() {
                            Some(i) => (i + 1) % self.experiments.len(),
                            None => 0,
                        };
                        self.experiments_list_state.select(Some(i));
                    }
                }
            }
            KeyCode::Enter => {
                if !self.show_jobs {
                    if let Some(selected) = self.experiments_list_state.selected() {
                        if let Some(experiment) = self.experiments.get(selected) {
                            let _ = self.event_tx.send(AppEvent::ExperimentsEvent(
                                ExperimentsEvent::ExperimentSelected(experiment.id.clone()),
                            ));
                        }
                    }
                }
            }
            KeyCode::Esc => {
                if self.show_jobs {
                    let _ = self.event_tx.send(AppEvent::ExperimentsEvent(
                        ExperimentsEvent::ShowExperimentJobs(false),
                    ));
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect) {
        if self.show_jobs {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            self.render_experiments_list(f, chunks[0]);
            self.render_experiment_jobs(f, chunks[1]);
        } else {
            self.render_experiments_list(f, area);
        }
    }

    fn title(&self) -> &str {
        "Experiments"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn on_navigation(&mut self, context: &NavigationContext) {
        // Handle navigation context - check for experiment selection
        if let Some(experiment_id) = &context.selection_id {
            self.selected_experiment = Some(experiment_id.clone());
            // Find and select the experiment in the list
            if let Some(pos) = self.experiments.iter().position(|e| e.id == *experiment_id) {
                self.experiments_list_state.select(Some(pos));
            }
        }
    }

    fn set_navigator(&mut self, navigator: TabNavigator) {
        self.navigator = Some(navigator);
    }
}

impl ExperimentsTab {
    fn render_experiments_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .experiments
            .iter()
            .map(|exp| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        &exp.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!(
                            "Jobs: {} | Created: {}",
                            exp.job_count,
                            exp.created_time.format("%Y-%m-%d %H:%M")
                        ),
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        let title = if self.loading {
            "Experiments (Loading...)"
        } else {
            "Experiments"
        };

        let experiments_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        f.render_stateful_widget(
            experiments_list,
            area,
            &mut self.experiments_list_state.clone(),
        );
    }

    fn render_experiment_jobs(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .experiment_jobs
            .iter()
            .map(|job| {
                let status_color = match job.status {
                    crate::azure::JobStatus::Running => Color::Yellow,
                    crate::azure::JobStatus::Completed => Color::Green,
                    crate::azure::JobStatus::Failed => Color::Red,
                    crate::azure::JobStatus::Canceled => Color::Gray,
                    _ => Color::White,
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            &job.name,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}]", job.status),
                            Style::default().fg(status_color),
                        ),
                    ]),
                    Line::from(Span::styled(
                        format!(
                            "Type: {} | Created: {}",
                            job.job_type,
                            job.created_time.format("%Y-%m-%d %H:%M")
                        ),
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        let title = if let Some(exp_id) = &self.selected_experiment {
            format!("Jobs in Experiment: {}", exp_id)
        } else {
            "Experiment Jobs".to_string()
        };

        let jobs_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        f.render_stateful_widget(jobs_list, area, &mut self.jobs_list_state.clone());
    }
}
