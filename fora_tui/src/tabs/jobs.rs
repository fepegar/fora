use super::Tab;
use crate::app::AppEvent;
use crate::navigation::{NavigationContext, TabNavigator};
use crate::{
    azure::{AzureClient, Job, JobDetails, JobStatus},
    cache::CacheManager,
};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum JobsEvent {
    JobSelected(String),
    JobCancelled(String),
    JobDetailsLoaded(String, JobDetails),
    ShowJobDetails(bool),
}

pub struct JobsTab {
    // State
    jobs: Vec<Job>,
    selected_job: Option<String>,
    job_details: Option<JobDetails>,
    show_details: bool,
    loading: bool,

    // UI state
    list_state: ListState,
    details_scroll_state: ScrollbarState,
    details_scroll_position: usize,

    // Dependencies
    azure_client: AzureClient,
    cache: CacheManager,
    event_tx: UnboundedSender<AppEvent>,
    navigator: Option<TabNavigator>,
}

impl JobsTab {
    pub fn new(
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Self {
        Self {
            jobs: Vec::new(),
            selected_job: None,
            job_details: None,
            show_details: false,
            loading: false,
            list_state: ListState::default(),
            details_scroll_state: ScrollbarState::default(),
            details_scroll_position: 0,
            azure_client,
            cache,
            event_tx,
            navigator: None,
        }
    }

    pub async fn handle_event(&mut self, event: JobsEvent) {
        match event {
            JobsEvent::JobSelected(job_id) => {
                tracing::debug!("Job selected: {}", job_id);
                self.selected_job = Some(job_id.clone());
                self.show_details = true;
                self.load_job_details(job_id).await;
            }
            JobsEvent::JobCancelled(job_id) => {
                self.cancel_job(job_id).await;
            }
            JobsEvent::JobDetailsLoaded(job_id, details) => {
                tracing::debug!("Received job details for: {}", job_id);
                if Some(&job_id) == self.selected_job.as_ref() {
                    tracing::debug!("Setting job details for selected job: {}", job_id);
                    self.job_details = Some(details);
                    self.details_scroll_position = 0; // Reset scroll when new details load
                    self.details_scroll_state = ScrollbarState::default();
                } else {
                    tracing::debug!("Ignoring job details for non-selected job: {}", job_id);
                }
            }
            JobsEvent::ShowJobDetails(show) => {
                self.show_details = show;
                if !show {
                    self.selected_job = None;
                    self.job_details = None;
                    self.details_scroll_position = 0; // Reset scroll when hiding details
                    self.details_scroll_state = ScrollbarState::default();
                }
            }
        }
    }

    async fn load_jobs(&mut self) {
        tracing::debug!("Loading jobs - current jobs count: {}", self.jobs.len());

        // Always show cached data immediately if available (even if expired)
        let cached_jobs = self.cache.get_jobs_cached();
        if !cached_jobs.is_empty() && self.jobs.is_empty() {
            self.jobs = cached_jobs;
            // Initialize list state with first item selected
            if self.list_state.selected().is_none() && !self.jobs.is_empty() {
                self.list_state.select(Some(0));
            }
            tracing::debug!(
                "Loaded {} jobs from cache for immediate display",
                self.jobs.len()
            );
        }

        // Then refresh in background
        self.load_jobs_background().await;

        // If we had cached data, we're not really loading from user's perspective
        if !self.jobs.is_empty() {
            self.loading = false;
        }
    }

    async fn load_jobs_background(&mut self) {
        // Start loading fresh data in background
        self.loading = true;
        let client = self.azure_client.clone();
        let cache = self.cache.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            tracing::debug!("Fetching fresh jobs from Azure");
            match client.get_recent_jobs().await {
                Ok(jobs) => {
                    tracing::debug!("Fetched {} jobs from Azure", jobs.len());
                    cache.store_jobs(jobs.clone()).await;
                    // Send refresh event to update UI with new data
                    let _ = tx.send(AppEvent::Refresh);
                }
                Err(e) => {
                    tracing::error!("Failed to load jobs: {}", e);
                }
            }
        });
    }

    async fn load_job_details(&mut self, job_id: String) {
        tracing::debug!("Loading job details synchronously for: {}", job_id);

        // Load job details synchronously to avoid hanging
        match self.azure_client.get_job_details(&job_id).await {
            Ok(details) => {
                tracing::debug!("Successfully loaded job details for: {}", job_id);
                if Some(&job_id) == self.selected_job.as_ref() {
                    self.job_details = Some(details);
                    self.details_scroll_position = 0;
                    self.details_scroll_state = ScrollbarState::default();
                }
            }
            Err(e) => {
                tracing::error!("Failed to load job details for {}: {}", job_id, e);
            }
        }
    }

    async fn cancel_job(&mut self, job_id: String) {
        let client = self.azure_client.clone();

        tokio::spawn(async move {
            match client.cancel_job(&job_id).await {
                Ok(_) => {
                    tracing::info!("Job {} cancelled successfully", job_id);
                }
                Err(e) => {
                    tracing::error!("Failed to cancel job {}: {}", job_id, e);
                }
            }
        });
    }

    fn next_job(&mut self) {
        if !self.jobs.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => (i + 1) % self.jobs.len(),
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn previous_job(&mut self) {
        if !self.jobs.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.jobs.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn select_current_job(&mut self) {
        tracing::debug!(
            "Selecting current job - list state: {:?}, jobs count: {}",
            self.list_state.selected(),
            self.jobs.len()
        );
        if let Some(selected) = self.list_state.selected() {
            if let Some(job) = self.jobs.get(selected) {
                tracing::debug!("Selecting job: {} ({})", job.name, job.id);
                let _ = self
                    .event_tx
                    .send(AppEvent::JobsEvent(JobsEvent::JobSelected(job.id.clone())));
            } else {
                tracing::warn!("No job at selected index: {}", selected);
            }
        } else {
            tracing::warn!("No job selected in list state");
        }
    }
}

#[async_trait]
impl Tab for JobsTab {
    async fn initialize(&mut self) {
        // Skip if we already have data loaded to avoid redundant work
        if !self.jobs.is_empty() {
            tracing::debug!(
                "Jobs tab already has data ({} jobs), skipping initialization",
                self.jobs.len()
            );
            return;
        }

        tracing::debug!("Initializing Jobs tab - no existing data");

        // Load cached data first for immediate display (even if expired)
        let cached_jobs = self.cache.get_jobs_cached();
        if !cached_jobs.is_empty() {
            self.jobs = cached_jobs;
            // Initialize list state with first item selected
            if self.list_state.selected().is_none() && !self.jobs.is_empty() {
                self.list_state.select(Some(0));
            }
            tracing::debug!("Loaded {} jobs from cache", self.jobs.len());
        } else {
            // If no cached data, load jobs synchronously to prevent empty state
            tracing::debug!("No cached jobs found, loading fresh data synchronously");
            match self.azure_client.get_recent_jobs().await {
                Ok(jobs) => {
                    tracing::debug!("Loaded {} jobs directly from Azure", jobs.len());
                    self.jobs = jobs.clone();
                    self.cache.store_jobs(jobs).await;
                    // Initialize list state with first item selected
                    if !self.jobs.is_empty() {
                        self.list_state.select(Some(0));
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load jobs during initialization: {}", e);
                }
            }
        }

        // Then refresh in background for updates
        self.load_jobs_background().await;
    }

    async fn refresh(&mut self) {
        // Always refresh from cache first for immediate update
        let cached_jobs = self.cache.get_jobs_cached();
        tracing::debug!("Refreshing jobs - found {} cached jobs", cached_jobs.len());
        if !cached_jobs.is_empty() {
            self.jobs = cached_jobs;
            // Initialize list state with first item selected if none selected
            if self.list_state.selected().is_none() && !self.jobs.is_empty() {
                self.list_state.select(Some(0));
            }
        }

        // Stop loading state if we have fresh data
        self.loading = false;

        // Only reload if we don't have any data
        if self.jobs.is_empty() {
            tracing::debug!("No jobs available, loading fresh data");
            self.load_jobs_background().await;
        }
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.show_details {
                    self.scroll_details_up();
                } else {
                    self.previous_job();
                }
            }
            KeyCode::Down => {
                if self.show_details {
                    self.scroll_details_down();
                } else {
                    self.next_job();
                }
            }
            KeyCode::Left => {
                if self.show_details {
                    self.previous_job();
                }
            }
            KeyCode::Right => {
                if self.show_details {
                    self.next_job();
                }
            }
            KeyCode::Enter => self.select_current_job(),
            KeyCode::Esc => {
                if self.show_details {
                    let _ = self
                        .event_tx
                        .send(AppEvent::JobsEvent(JobsEvent::ShowJobDetails(false)));
                }
            }
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                if let Some(job_id) = &self.selected_job {
                    let _ = self
                        .event_tx
                        .send(AppEvent::JobsEvent(JobsEvent::JobCancelled(job_id.clone())));
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect) {
        if self.show_details {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);

            self.render_jobs_list(f, chunks[0]);
            self.render_job_details(f, chunks[1]);
        } else {
            self.render_jobs_list(f, area);
        }
    }

    fn title(&self) -> &str {
        "Jobs"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn on_navigation(&mut self, context: &NavigationContext) {
        // Handle navigation context - check for job selection
        if let Some(job_id) = &context.selection_id {
            self.selected_job = Some(job_id.clone());
            // Find and select the job in the list
            if let Some(pos) = self.jobs.iter().position(|j| j.id == *job_id) {
                self.list_state.select(Some(pos));
                // Load job details
                self.load_job_details(job_id.clone()).await;
            }
        }
    }

    fn set_navigator(&mut self, navigator: TabNavigator) {
        self.navigator = Some(navigator);
    }
}

impl JobsTab {
    fn scroll_details_up(&mut self) {
        if self.details_scroll_position > 0 {
            self.details_scroll_position -= 1;
            self.details_scroll_state = self
                .details_scroll_state
                .position(self.details_scroll_position);
        }
    }

    fn scroll_details_down(&mut self) {
        self.details_scroll_position += 1;
        self.details_scroll_state = self
            .details_scroll_state
            .position(self.details_scroll_position);
    }
    fn render_jobs_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .jobs
            .iter()
            .map(|job| {
                let status_color = match job.status {
                    JobStatus::Running => Color::Yellow,
                    JobStatus::Completed => Color::Green,
                    JobStatus::Failed => Color::Red,
                    JobStatus::Canceled => Color::Gray,
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
                            "Experiment: {} | Type: {}",
                            job.experiment_name, job.job_type
                        ),
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        // Different border configurations based on whether details pane is shown
        let border_set = if self.show_details {
            // When details pane is open, don't show right border and use connecting corners
            let connecting_border_set = symbols::border::Set {
                top_left: symbols::line::VERTICAL_RIGHT,
                top_right: symbols::line::HORIZONTAL_DOWN,
                bottom_right: symbols::line::HORIZONTAL_UP,
                ..symbols::border::ROUNDED
            };
            connecting_border_set
        } else {
            // When details pane is closed, show all borders with normal corners
            let normal_border_set = symbols::border::Set {
                top_left: symbols::line::VERTICAL_RIGHT,
                top_right: symbols::line::VERTICAL_LEFT,
                ..symbols::border::ROUNDED
            };
            normal_border_set
        };

        let jobs_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border_set)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(jobs_list, area, &mut self.list_state.clone());
    }

    fn render_job_details(&mut self, f: &mut Frame, area: Rect) {
        // TODO: Share this somewhere
        let top_border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::ROUNDED
        };

        if let Some(details) = &self.job_details {
            // Create the scrollable content area and scrollbar area
            let scrollable_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width, // .saturating_sub(1)
                height: area.height,
            };
            let scrollbar_area = Rect {
                x: area.right().saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            let details_text = vec![
                Line::from(vec![
                    Span::styled("Job ID: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&details.id),
                ]),
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&details.name),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}", details.status)),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&details.job_type),
                ]),
                Line::from(vec![
                    Span::styled("Experiment: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&details.experiment_name),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Command:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(details.command.as_deref().unwrap_or("N/A")),
                Line::from(""),
                Line::from(Span::styled(
                    "Environment:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(details.environment.as_deref().unwrap_or("N/A")),
                Line::from(""),
                Line::from(Span::styled(
                    "Compute Target:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(details.compute_target.as_deref().unwrap_or("N/A")),
                Line::from(""),
                Line::from(Span::styled(
                    "Created: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(
                    details
                        .created_time
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string(),
                ),
                Line::from(""),
                Line::from(Span::styled(
                    "Started: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(details.start_time.map_or("N/A".to_string(), |t| {
                    t.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                })),
                Line::from(""),
                Line::from(Span::styled(
                    "Ended: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(details.end_time.map_or("N/A".to_string(), |t| {
                    t.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                })),
            ];

            let content_length = details_text.len();
            let visible_height = scrollable_area.height.saturating_sub(2) as usize; // Account for borders
            let max_scroll = content_length.saturating_sub(visible_height);

            // Clamp scroll position
            if self.details_scroll_position > max_scroll {
                self.details_scroll_position = max_scroll;
            }

            // Update scrollbar state
            self.details_scroll_state = self
                .details_scroll_state
                .content_length(content_length)
                .position(self.details_scroll_position);

            let paragraph = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                        .border_set(top_border_set)
                        .border_style(Style::default().fg(Color::Gray)),
                )
                .wrap(Wrap { trim: true })
                .scroll((self.details_scroll_position as u16, 0));

            f.render_widget(paragraph, scrollable_area);

            // Render scrollbar if content is scrollable
            if content_length > visible_height {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));

                f.render_stateful_widget(scrollbar, scrollbar_area, &mut self.details_scroll_state);
            }
        } else {
            let loading_text = if self.selected_job.is_some() {
                "⏳ Loading job details..."
            } else {
                "Select a job to view details"
            };

            let paragraph = Paragraph::new(loading_text)
                .block(
                    Block::default()
                        .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                        .border_set(top_border_set)
                        .border_style(Style::default().fg(Color::Gray)),
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, area);
        }
    }
}
