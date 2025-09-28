use crate::{
    azure::AzureClient, cache::CacheManager, navigation::NavigationManager,
    tab_registry::TabRegistry, tabs::*,
};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    TabChanged(TabType),
    TabInitialized(TabType),
    Refresh,
    KeyPress(crossterm::event::KeyEvent),
    Navigate(String, TabType), // Simplified: (action_type, target_tab)
    // Navigation events
    NextTab,
    PreviousTab,
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Help,
    Key(char),
    // Tab-specific events
    HomeEvent(home::HomeEvent),
    JobsEvent(jobs::JobsEvent),
    ExperimentsEvent(experiments::ExperimentsEvent),
    ComputeEvent(compute::ComputeEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabType {
    Home,
    Jobs,
    Experiments,
    Compute,
}

impl std::fmt::Display for TabType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabType::Home => write!(f, "Home"),
            TabType::Jobs => write!(f, "Jobs"),
            TabType::Experiments => write!(f, "Experiments"),
            TabType::Compute => write!(f, "Compute"),
        }
    }
}

#[derive(Debug)]
pub enum AppState {
    Loading,
    Ready,
    Error(String),
}

pub struct App {
    pub state: AppState,
    pub current_tab: TabType,
    pub tabs: Vec<TabType>,

    // Tab instances
    tab_instances: HashMap<TabType, Box<dyn Tab>>,
    tab_registry: TabRegistry,

    // Shared resources
    azure_client: AzureClient,
    cache: CacheManager,
    event_tx: UnboundedSender<AppEvent>,

    // Navigation
    navigation: NavigationManager,

    // Background tasks
    background_tasks: tokio::task::JoinSet<()>,
}

impl App {
    pub async fn new(
        event_tx: UnboundedSender<AppEvent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let azure_client = AzureClient::new().await?;
        let cache = CacheManager::new();

        // Create tab registry and use it to create tabs
        let tab_registry = TabRegistry::new();
        let tabs = tab_registry.tab_order().to_vec();
        let tab_instances =
            tab_registry.create_all_tabs(azure_client.clone(), cache.clone(), event_tx.clone());

        let mut app = Self {
            state: AppState::Loading,
            current_tab: TabType::Home,
            tabs,
            tab_instances,
            tab_registry,
            azure_client,
            cache: cache.clone(),
            event_tx: event_tx.clone(),
            navigation: NavigationManager::new(event_tx),
            background_tasks: tokio::task::JoinSet::new(),
        };

        // Start periodic cache cleanup task
        app.start_cache_cleanup_task(cache);

        // Start background preloading for commonly accessed tabs
        app.start_background_preloading();

        // Initialize current tab
        app.initialize_current_tab().await;
        app.state = AppState::Ready;

        Ok(app)
    }

    pub async fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Navigate(action_type, tab_type) => match action_type.as_str() {
                "goto" => {
                    let _ = self.event_tx.send(AppEvent::TabChanged(tab_type));
                }
                "back" => {
                    if self.navigation.can_go_back() {
                        let history = self.navigation.history();
                        if history.len() > 1 {
                            let previous_tab = history[history.len() - 2].clone();
                            let _ = self.event_tx.send(AppEvent::TabChanged(previous_tab));
                        }
                    }
                }
                _ => {}
            },
            AppEvent::TabChanged(tab) => {
                self.current_tab = tab;
                self.initialize_current_tab().await;
            }
            AppEvent::TabInitialized(tab_type) => {
                // Initialize the tab in background after navigation is complete
                if tab_type == self.current_tab {
                    if let Some(tab) = self.tab_instances.get_mut(&tab_type) {
                        tab.initialize().await;
                    }
                }
            }
            AppEvent::KeyPress(key) => {
                self.handle_key_event(key).await;
            }
            AppEvent::Refresh => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.refresh().await;
                }
            }
            AppEvent::HomeEvent(home_event) => {
                if let Some(tab) = self.tab_instances.get_mut(&TabType::Home) {
                    if let Some(home_tab) = tab.as_any_mut().downcast_mut::<home::HomeTab>() {
                        home_tab.handle_event(home_event).await;
                    }
                }
            }
            AppEvent::JobsEvent(jobs_event) => {
                if let Some(tab) = self.tab_instances.get_mut(&TabType::Jobs) {
                    if let Some(jobs_tab) = tab.as_any_mut().downcast_mut::<jobs::JobsTab>() {
                        jobs_tab.handle_event(jobs_event).await;
                    }
                }
            }
            AppEvent::ExperimentsEvent(experiments_event) => {
                if let Some(tab) = self.tab_instances.get_mut(&TabType::Experiments) {
                    if let Some(experiments_tab) = tab
                        .as_any_mut()
                        .downcast_mut::<experiments::ExperimentsTab>()
                    {
                        experiments_tab.handle_event(experiments_event).await;
                    }
                }
            }
            AppEvent::ComputeEvent(compute_event) => {
                if let Some(tab) = self.tab_instances.get_mut(&TabType::Compute) {
                    if let Some(compute_tab) =
                        tab.as_any_mut().downcast_mut::<compute::ComputeTab>()
                    {
                        compute_tab.handle_event(compute_event).await;
                    }
                }
            }
            AppEvent::Up => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Up,
                        crossterm::event::KeyModifiers::empty(),
                    ))
                    .await;
                }
            }
            AppEvent::Down => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Down,
                        crossterm::event::KeyModifiers::empty(),
                    ))
                    .await;
                }
            }
            AppEvent::Left => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Left,
                        crossterm::event::KeyModifiers::empty(),
                    ))
                    .await;
                }
            }
            AppEvent::Right => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Right,
                        crossterm::event::KeyModifiers::empty(),
                    ))
                    .await;
                }
            }
            AppEvent::Select => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Enter,
                        crossterm::event::KeyModifiers::empty(),
                    ))
                    .await;
                }
            }
            AppEvent::Back => {
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Esc,
                        crossterm::event::KeyModifiers::empty(),
                    ))
                    .await;
                }
            }
            AppEvent::NextTab => {
                self.next_tab().await;
            }
            AppEvent::PreviousTab => {
                let current_index = self
                    .tabs
                    .iter()
                    .position(|t| *t == self.current_tab)
                    .unwrap_or(0);
                let prev_index = if current_index == 0 {
                    self.tabs.len() - 1
                } else {
                    current_index - 1
                };
                let prev_tab = self.tabs[prev_index].clone();
                let _ = self.event_tx.send(AppEvent::TabChanged(prev_tab));
            }
            AppEvent::Key(c) => {
                // Check if it's a tab shortcut first
                if let Some(tab_type) = self.tab_registry.tab_by_shortcut(c) {
                    let _ = self.event_tx.send(AppEvent::TabChanged(tab_type.clone()));
                } else {
                    // Forward key events to current tab
                    if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                        tab.handle_key(crossterm::event::KeyEvent::new(
                            crossterm::event::KeyCode::Char(c),
                            crossterm::event::KeyModifiers::empty(),
                        ))
                        .await;
                    }
                }
            }
            AppEvent::Help => {
                // Handle help display if needed
                // For now, just pass 'h' to tab shortcuts
                if let Some(tab_type) = self.tab_registry.tab_by_shortcut('h') {
                    let _ = self.event_tx.send(AppEvent::TabChanged(tab_type.clone()));
                }
            }
            _ => {}
        }
    }

    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('q') => {
                let _ = self.event_tx.send(AppEvent::Quit);
            }
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                let _ = self.event_tx.send(AppEvent::Quit);
            }
            KeyCode::Tab => {
                self.next_tab().await;
            }
            KeyCode::Char('r') => {
                let _ = self.event_tx.send(AppEvent::Refresh);
            }
            KeyCode::Char(c) => {
                // Check if it's a tab shortcut first
                if let Some(tab_type) = self.tab_registry.tab_by_shortcut(c) {
                    let _ = self.event_tx.send(AppEvent::TabChanged(tab_type.clone()));
                } else {
                    // Forward key events to current tab
                    if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                        tab.handle_key(key).await;
                    }
                }
            }
            _ => {
                // Forward key events to current tab
                if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
                    tab.handle_key(key).await;
                }
            }
        }
    }

    async fn next_tab(&mut self) {
        let current_index = self
            .tabs
            .iter()
            .position(|t| *t == self.current_tab)
            .unwrap_or(0);
        let next_index = (current_index + 1) % self.tabs.len();
        let next_tab = self.tabs[next_index].clone();
        let _ = self.event_tx.send(AppEvent::TabChanged(next_tab));
    }

    async fn initialize_current_tab(&mut self) {
        if let Some(tab) = self.tab_instances.get_mut(&self.current_tab) {
            // Provide navigation context first (non-blocking)
            let context = self.navigation.context();
            tab.on_navigation(context).await;

            // Initialize in background to avoid blocking navigation
            let tab_type = self.current_tab.clone();
            let event_tx = self.event_tx.clone();

            // Spawn background initialization
            tokio::spawn(async move {
                // Send a refresh event to trigger background loading
                let _ = event_tx.send(AppEvent::TabInitialized(tab_type));
            });
        }
    }

    fn start_cache_cleanup_task(&mut self, cache: CacheManager) {
        // Spawn background task for periodic cache cleanup
        self.background_tasks.spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                cache.clear_expired();
                tracing::debug!("Cache cleanup completed");
            }
        });
    }

    fn start_background_preloading(&mut self) {
        // Preload Jobs and Experiments tabs in background since they're commonly accessed
        let event_tx = self.event_tx.clone();

        // Spawn background preloading task
        self.background_tasks.spawn(async move {
            // Wait a bit to let the initial UI load
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Preload Jobs tab
            let _ = event_tx.send(AppEvent::TabInitialized(TabType::Jobs));

            // Wait between preloads to not overwhelm the system
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            // Preload Experiments tab
            let _ = event_tx.send(AppEvent::TabInitialized(TabType::Experiments));

            tracing::debug!("Background preloading completed");
        });
    }

    pub fn get_current_tab(&mut self) -> Option<&mut (dyn Tab + '_)> {
        match self.tab_instances.get_mut(&self.current_tab) {
            Some(tab) => Some(tab.as_mut()),
            None => None,
        }
    }

    pub fn get_current_tab_mut(&mut self) -> Option<&mut (dyn Tab + '_)> {
        match self.tab_instances.get_mut(&self.current_tab) {
            Some(tab) => Some(tab.as_mut()),
            None => None,
        }
    }
}
