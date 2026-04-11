use std::io;
use std::time::Duration;

use anyhow::Result;
use azure_ml::models::JobStatus;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::client::AzureClient;
use crate::components::help_bar;
use crate::components::tab_bar;
use crate::components::workspace_picker::{centered_rect, WorkspacePicker};
use crate::config::AppConfig;
use crate::event::{Event, EventHandler};
use crate::experiment_cache::ExperimentDiskCache;
use crate::tabs::compute::ComputeTab;
use crate::tabs::experiments::ExperimentsTab;
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::tabs::recent_jobs::RecentJobsTab;
use crate::tabs::Tab;
use crate::theme::Theme;

use crate::tabs::compute::state::ComputeRow;
use std::collections::HashMap;

/// Actions dispatched by tabs and components back to the app.
#[derive(Debug, Clone)]
pub enum Action {
    // Recent Jobs tab
    RecentJobsBatchLoaded(Vec<RecentJobRow>),
    RecentJobsFetchComplete,
    RecentJobEnriched {
        job_id: String,
        compute_target: Option<String>,
        job_type: Option<String>,
        command: Option<String>,
        environment_id: Option<String>,
        description: Option<String>,
        tags: HashMap<String, String>,
        status: Option<JobStatus>,
        end_time: Option<DateTime<Utc>>,
    },

    // Incremental refresh for Recent Jobs
    RecentJobsIncrementalBatch(Vec<RecentJobRow>),

    // Pending/scheduled jobs (fetched separately to avoid pagination through all jobs)
    RecentJobsPendingBatch(Vec<RecentJobRow>),

    // Start time updated for a job that transitioned from pending to running
    RecentJobStartTimeUpdated {
        job_id: String,
        start_time: DateTime<Utc>,
    },

    // Experiment cache (shared by Recent Jobs and Experiments tabs)
    ExperimentCacheUpdated(HashMap<String, String>),

    // Experiments tab
    ExperimentsDiscovered(Vec<(String, String, Option<DateTime<Utc>>)>),
    ExperimentDiscoveryProgress {
        completed: usize,
        total: usize,
    },
    ExperimentDiscoveryComplete,
    ExperimentJobsLoaded {
        experiment_id: String,
        jobs: Vec<RecentJobRow>,
    },

    // Incremental refresh for Experiments
    ExperimentIncrementalUpdate(Vec<(String, String, Option<DateTime<Utc>>)>),
    ExperimentIncrementalComplete,

    // Compute tab
    ComputeLoaded(Vec<ComputeRow>),

    // Metric keys updated for an existing run (e.g. new metrics logged after initial fetch)
    MetricKeysUpdated {
        job_id: String,
        metric_keys: Vec<String>,
    },

    // Metrics (detail pane)
    MetricBatchLoaded {
        run_id: String,
        metric: (String, Vec<(f64, f64)>),
    },
    MetricsFetchComplete {
        run_id: String,
    },
    MetricsFetchFailed {
        run_id: String,
        error: String,
    },

    // Shared
    Error(String),
    RefreshRequested,
    SaveColumnConfig {
        tab: String,
        columns: Vec<String>,
    },
}

pub struct App {
    config: AppConfig,
    username: String,
    tabs: Vec<Box<dyn Tab>>,
    active_tab: usize,
    workspace_picker: WorkspacePicker,
    show_help_bar: bool,
    show_no_config: bool,
    error_message: Option<String>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    should_quit: bool,
    active_workspace_idx: Option<usize>,
    experiment_disk_cache: Option<ExperimentDiskCache>,
}

impl App {
    pub async fn new(config: AppConfig, username: String) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let show_help_bar = config.ui.show_help_bar;
        let refresh_interval = config.ui.refresh_interval_secs;
        let show_no_config = config.workspaces.is_empty();

        // Determine initial workspace index based on default_workspace config
        let initial_ws_idx = config
            .default_workspace
            .as_ref()
            .and_then(|name| config.workspaces.iter().position(|ws| ws.name == *name))
            .unwrap_or(0);

        // Try to create a client from the initial workspace if available
        let client = config.workspaces.get(initial_ws_idx).and_then(|ws| {
            AzureClient::new(ws.clone())
                .map_err(|e| tracing::warn!("Failed to create Azure client: {}", e))
                .ok()
        });

        let active_workspace_idx = if client.is_some() {
            Some(initial_ws_idx)
        } else {
            None
        };

        // Load experiment cache from disk for the active workspace
        let experiment_disk_cache = config.workspaces.get(initial_ws_idx).map(|ws| {
            ExperimentDiskCache::load(&ws.subscription_id, &ws.resource_group, &ws.workspace_name)
        });
        let initial_exp_cache = experiment_disk_cache
            .as_ref()
            .map(|c| c.experiments.clone())
            .unwrap_or_default();

        let tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(RecentJobsTab::new(
                client.clone(),
                username.clone(),
                config.columns.jobs.as_deref(),
                initial_exp_cache.clone(),
            )),
            Box::new(ExperimentsTab::new(client.clone(), initial_exp_cache)),
            Box::new(ComputeTab::new(
                client,
                refresh_interval,
                config.columns.compute.as_deref(),
            )),
        ];

        let workspace_picker = WorkspacePicker::new(config.workspaces.clone());

        Ok(Self {
            config,
            username,
            tabs,
            active_tab: 0,
            workspace_picker,
            show_help_bar,
            show_no_config,
            error_message: None,
            action_tx,
            action_rx,
            should_quit: false,
            active_workspace_idx,
            experiment_disk_cache,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let tick_rate = Duration::from_millis(250);
        let mut events = EventHandler::new(tick_rate);

        // Main loop
        loop {
            // Draw
            terminal.draw(|frame| self.render(frame))?;

            // Handle events
            if let Some(event) = events.next().await {
                match event {
                    Event::Key(key) => self.handle_key(key),
                    Event::Tick => self.handle_tick(),
                    Event::Resize(_, _) => {} // ratatui handles resize automatically
                }
            }

            // Process pending actions
            while let Ok(action) = self.action_rx.try_recv() {
                self.handle_action(action);
            }

            if self.should_quit {
                break;
            }
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.size();

        // Layout: tab bar (1) | content (flex) | help bar (1, optional) | error (1, optional)
        let mut constraints = vec![Constraint::Length(1), Constraint::Min(1)];

        if self.show_help_bar {
            constraints.push(Constraint::Length(1));
        }
        if self.error_message.is_some() {
            constraints.push(Constraint::Length(1));
        }

        let chunks = Layout::vertical(constraints).split(area);

        // Tab bar
        let tab_titles: Vec<&str> = self.tabs.iter().map(|t| t.title()).collect();
        tab_bar::render_tab_bar(frame, chunks[0], &tab_titles, self.active_tab);

        // Content
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.render(frame, chunks[1]);
        }

        let mut idx = 2;

        // Help bar
        if self.show_help_bar {
            let mut hints: Vec<(&str, &str)> = vec![("Tab", "Switch Tab"), ("w", "Workspace")];
            if let Some(tab) = self.tabs.get(self.active_tab) {
                hints.extend(tab.key_hints());
            }
            hints.push(("?", "Help"));
            hints.push(("q", "Quit"));
            help_bar::render_help_bar(frame, chunks[idx], &hints);
            idx += 1;
        }

        // Error bar
        if let Some(ref err) = self.error_message {
            let error =
                Paragraph::new(format!(" ⚠ {}", err)).style(Style::default().fg(Theme::ERROR));
            frame.render_widget(error, chunks[idx]);
        }

        // Overlay modals
        self.workspace_picker.render(frame, area);

        if self.show_no_config {
            render_no_config_popup(frame, area);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // No-config popup: any key dismisses and quits
        if self.show_no_config {
            self.should_quit = true;
            return;
        }

        // Clear error on any key press
        self.error_message = None;

        // Workspace picker takes priority
        if self.workspace_picker.active {
            if let Some(idx) = self.workspace_picker.handle_key(key) {
                self.switch_workspace(idx);
            }
            return;
        }

        // Tab-level key handling
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if tab.handle_key(key, &self.action_tx) {
                return;
            }
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % self.tabs.len();
            }
            KeyCode::BackTab => {
                self.active_tab = if self.active_tab == 0 {
                    self.tabs.len() - 1
                } else {
                    self.active_tab - 1
                };
            }
            KeyCode::Char('w') => {
                self.workspace_picker.open();
            }
            KeyCode::Char('?') => {
                self.show_help_bar = !self.show_help_bar;
            }
            _ => {}
        }
    }

    fn handle_tick(&mut self) {
        if self.show_no_config {
            return;
        }
        for tab in &mut self.tabs {
            tab.tick(&self.action_tx);
        }
    }

    fn handle_action(&mut self, action: Action) {
        match &action {
            Action::Error(msg) => {
                self.error_message = Some(msg.clone());
            }
            Action::RefreshRequested => {
                // Re-trigger tick to pick up invalidated caches
                self.handle_tick();
            }
            Action::SaveColumnConfig { tab, columns } => {
                match tab.as_str() {
                    "recent_jobs" => {
                        self.config.columns.jobs = Some(columns.clone());
                    }
                    "compute" => {
                        self.config.columns.compute = Some(columns.clone());
                    }
                    _ => {}
                }
                if let Err(e) = self.config.save_columns() {
                    tracing::warn!("Failed to save column config: {}", e);
                }
            }
            Action::ExperimentCacheUpdated(cache) => {
                if let Some(ref mut disk_cache) = self.experiment_disk_cache {
                    disk_cache.save(cache);
                }
            }
            _ => {}
        }

        // Forward to all tabs
        for tab in &mut self.tabs {
            tab.update(&action);
        }
    }

    fn switch_workspace(&mut self, idx: usize) {
        if let Some(ws) = self.config.workspaces.get(idx) {
            match AzureClient::new(ws.clone()) {
                Ok(client) => {
                    self.active_workspace_idx = Some(idx);

                    // Load experiment cache for the new workspace
                    let disk_cache = ExperimentDiskCache::load(
                        &ws.subscription_id,
                        &ws.resource_group,
                        &ws.workspace_name,
                    );
                    let exp_cache = disk_cache.experiments.clone();
                    self.experiment_disk_cache = Some(disk_cache);

                    // Recreate tabs with new client
                    let refresh = self.config.ui.refresh_interval_secs;
                    self.tabs = vec![
                        Box::new(RecentJobsTab::new(
                            Some(client.clone()),
                            self.username.clone(),
                            self.config.columns.jobs.as_deref(),
                            exp_cache.clone(),
                        )),
                        Box::new(ExperimentsTab::new(Some(client.clone()), exp_cache)),
                        Box::new(ComputeTab::new(
                            Some(client),
                            refresh,
                            self.config.columns.compute.as_deref(),
                        )),
                    ];

                    tracing::info!("Switched to workspace: {}", ws.name);
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to connect to workspace: {}", e));
                }
            }
        }
    }
}

fn render_no_config_popup(frame: &mut ratatui::Frame, area: Rect) {
    let modal = centered_rect(60, 50, area);
    frame.render_widget(Clear, modal);

    let block = Block::default()
        .title(" No Workspaces Configured ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::WARNING))
        .style(Style::default().bg(Theme::MODAL_BG));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No workspaces found in configuration.",
            Style::default().fg(Theme::FG),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Run the following command to set up:",
            Style::default().fg(Theme::DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  fora init",
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This will discover your Azure ML workspaces",
            Style::default().fg(Theme::DIM),
        )),
        Line::from(Span::styled(
            "and create a configuration file.",
            Style::default().fg(Theme::DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to exit.",
            Style::default().fg(Theme::DIM),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, modal);
}
