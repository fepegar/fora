use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::execute;
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
use crate::tabs::compute::ComputeTab;
use crate::tabs::jobs::JobsTab;
use crate::tabs::Tab;
use crate::theme::Theme;

use crate::tabs::jobs::state::JobRow;
use crate::tabs::compute::state::ComputeRow;

/// Actions dispatched by tabs and components back to the app.
#[derive(Debug, Clone)]
pub enum Action {
    JobsBatchLoaded(Vec<JobRow>),
    JobsFetchPaused,
    JobsFetchComplete,
    JobsUpdated(Vec<JobRow>),
    JobsNewPrepended(Vec<JobRow>),
    ComputeLoaded(Vec<ComputeRow>),
    JobCancelled(String),
    Error(String),
    RefreshRequested,
    SaveColumnConfig {
        tab: String,
        columns: Vec<String>,
    },
}

pub struct App {
    config: AppConfig,
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
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let show_help_bar = config.ui.show_help_bar;
        let refresh_interval = config.ui.refresh_interval_secs;
        let show_no_config = config.workspaces.is_empty();

        // Try to create a client from the first workspace if available
        let client = config.workspaces.first().and_then(|ws| {
            AzureClient::new(ws.clone())
                .map_err(|e| tracing::warn!("Failed to create Azure client: {}", e))
                .ok()
        });

        let active_workspace_idx = if client.is_some() { Some(0) } else { None };

        let tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(JobsTab::new(client.clone(), refresh_interval, config.columns.jobs.as_deref())),
            Box::new(ComputeTab::new(client, refresh_interval, config.columns.compute.as_deref())),
        ];

        let workspace_picker = WorkspacePicker::new(config.workspaces.clone());

        Ok(Self {
            config,
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
            let error = Paragraph::new(format!(" ⚠ {}", err))
                .style(Style::default().fg(Theme::ERROR));
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
                    "jobs" => {
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

                    // Recreate tabs with new client
                    let refresh = self.config.ui.refresh_interval_secs;
                    self.tabs = vec![
                        Box::new(JobsTab::new(Some(client.clone()), refresh, self.config.columns.jobs.as_deref())),
                        Box::new(ComputeTab::new(Some(client), refresh, self.config.columns.compute.as_deref())),
                    ];

                    tracing::info!("Switched to workspace: {}", ws.name);
                }
                Err(e) => {
                    self.error_message =
                        Some(format!("Failed to connect to workspace: {}", e));
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

    let config_path = crate::config::AppConfig::global_config_path();

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No workspaces found in configuration.",
            Style::default().fg(Theme::FG),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Create a config file at:",
            Style::default().fg(Theme::DIM),
        )),
        Line::from(Span::styled(
            format!("  {}", config_path),
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  or ./fora.toml",
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Example:",
            Style::default().fg(Theme::DIM),
        )),
        Line::from(Span::styled(
            "  [[workspaces]]",
            Style::default().fg(Theme::FG),
        )),
        Line::from(Span::styled(
            "  name = \"my-workspace\"",
            Style::default().fg(Theme::FG),
        )),
        Line::from(Span::styled(
            "  subscription_id = \"xxxx-xxxx-xxxx\"",
            Style::default().fg(Theme::FG),
        )),
        Line::from(Span::styled(
            "  resource_group = \"my-rg\"",
            Style::default().fg(Theme::FG),
        )),
        Line::from(Span::styled(
            "  workspace_name = \"my-workspace\"",
            Style::default().fg(Theme::FG),
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
