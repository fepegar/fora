use crate::app::TabType;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

/// Generic navigation action that can be sent between tabs
#[derive(Debug)]
pub enum NavigationAction {
    /// Navigate to a specific tab
    GoToTab(TabType),
    /// Navigate to a tab and select a specific item by ID
    GoToTabWithSelection(TabType, String),
    /// Navigate to a tab with custom data
    GoToTabWithData(TabType, String), // Simplified to just use string data
    /// Go back to previous tab
    GoBack,
    /// Navigate to tab with query parameters
    GoToTabWithQuery(TabType, HashMap<String, String>),
}

/// Navigation context passed to tabs for navigation operations
#[derive(Debug, Clone)]
pub struct NavigationContext {
    pub current_tab: TabType,
    pub previous_tab: Option<TabType>,
    pub selection_id: Option<String>,
    pub query_params: HashMap<String, String>,
    pub custom_data: Option<String>,
}

impl Default for NavigationContext {
    fn default() -> Self {
        Self {
            current_tab: TabType::Home,
            previous_tab: None,
            selection_id: None,
            query_params: HashMap::new(),
            custom_data: None,
        }
    }
}

/// Navigator trait for tabs that need to navigate
pub trait Navigator {
    fn navigate(&self, action: NavigationAction);
}

/// Central navigation manager
pub struct NavigationManager {
    event_tx: UnboundedSender<crate::app::AppEvent>,
    context: NavigationContext,
    history: Vec<TabType>,
    custom_data_store: HashMap<String, String>,
}

impl NavigationManager {
    pub fn new(event_tx: UnboundedSender<crate::app::AppEvent>) -> Self {
        Self {
            event_tx,
            context: NavigationContext::default(),
            history: vec![TabType::Home],
            custom_data_store: HashMap::new(),
        }
    }

    /// Execute a navigation action
    pub fn navigate(&mut self, action: NavigationAction) {
        match action {
            NavigationAction::GoToTab(tab_type) => {
                self.go_to_tab(tab_type, None, HashMap::new(), None);
            }
            NavigationAction::GoToTabWithSelection(tab_type, selection_id) => {
                self.go_to_tab(tab_type, Some(selection_id), HashMap::new(), None);
            }
            NavigationAction::GoToTabWithData(tab_type, data) => {
                let key = format!("{:?}", tab_type);
                self.custom_data_store.insert(key.clone(), data);
                self.go_to_tab(tab_type, None, HashMap::new(), Some(key));
            }
            NavigationAction::GoBack => {
                if self.history.len() > 1 {
                    self.history.pop(); // Remove current
                    if let Some(previous_tab) = self.history.last() {
                        self.go_to_tab(*previous_tab, None, HashMap::new(), None);
                    }
                }
            }
            NavigationAction::GoToTabWithQuery(tab_type, query_params) => {
                self.go_to_tab(tab_type, None, query_params, None);
            }
        }
    }

    fn go_to_tab(
        &mut self,
        tab_type: TabType,
        selection_id: Option<String>,
        query_params: HashMap<String, String>,
        custom_data: Option<String>,
    ) {
        let previous_tab = Some(self.context.current_tab.clone());

        // Update history if this is a new tab (not going back)
        if self.history.last() != Some(&tab_type) {
            self.history.push(tab_type.clone());
        }

        // Update context
        self.context = NavigationContext {
            current_tab: tab_type.clone(),
            previous_tab,
            selection_id,
            query_params,
            custom_data,
        };

        // Send tab change event
        let _ = self
            .event_tx
            .send(crate::app::AppEvent::TabChanged(tab_type));
    }

    /// Get the current navigation context
    pub fn context(&self) -> &NavigationContext {
        &self.context
    }

    /// Get custom data by key
    pub fn get_custom_data(&self, key: &str) -> Option<&String> {
        self.custom_data_store.get(key)
    }

    /// Take custom data by key (removes it from store)
    pub fn take_custom_data(&mut self, key: &str) -> Option<String> {
        self.custom_data_store.remove(key)
    }

    /// Clear navigation history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history.push(self.context.current_tab.clone());
    }

    /// Get navigation history
    pub fn history(&self) -> &[TabType] {
        &self.history
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        self.history.len() > 1
    }
}

/// Navigator implementation that can be injected into tabs
#[derive(Clone)]
pub struct TabNavigator {
    event_tx: UnboundedSender<crate::app::AppEvent>,
}

impl TabNavigator {
    pub fn new(event_tx: UnboundedSender<crate::app::AppEvent>) -> Self {
        Self { event_tx }
    }

    /// Navigate to jobs tab with optional job selection
    pub fn to_jobs(&self, _job_id: Option<String>) {
        let _ = self.event_tx.send(crate::app::AppEvent::Navigate(
            "goto".to_string(),
            TabType::Jobs,
        ));
    }

    /// Navigate to experiments tab with optional experiment selection
    pub fn to_experiments(&self, _experiment_id: Option<String>) {
        let _ = self.event_tx.send(crate::app::AppEvent::Navigate(
            "goto".to_string(),
            TabType::Experiments,
        ));
    }

    /// Navigate to compute tab
    pub fn to_compute(&self) {
        let _ = self.event_tx.send(crate::app::AppEvent::Navigate(
            "goto".to_string(),
            TabType::Compute,
        ));
    }

    /// Navigate to home tab
    pub fn to_home(&self) {
        let _ = self.event_tx.send(crate::app::AppEvent::Navigate(
            "goto".to_string(),
            TabType::Home,
        ));
    }

    /// Go back to previous tab
    pub fn go_back(&self) {
        let _ = self.event_tx.send(crate::app::AppEvent::Navigate(
            "back".to_string(),
            TabType::Home,
        ));
    }

    /// Navigate with custom data
    pub fn to_tab_with_data(&self, tab_type: TabType, _data: String) {
        let _ = self
            .event_tx
            .send(crate::app::AppEvent::Navigate("goto".to_string(), tab_type));
    }

    /// Navigate with query parameters
    pub fn to_tab_with_query(&self, tab_type: TabType, _params: HashMap<String, String>) {
        let _ = self
            .event_tx
            .send(crate::app::AppEvent::Navigate("goto".to_string(), tab_type));
    }
}

impl Navigator for TabNavigator {
    fn navigate(&self, action: NavigationAction) {
        match action {
            NavigationAction::GoToTab(tab) => {
                let _ = self
                    .event_tx
                    .send(crate::app::AppEvent::Navigate("goto".to_string(), tab));
            }
            NavigationAction::GoToTabWithSelection(tab, _) => {
                let _ = self
                    .event_tx
                    .send(crate::app::AppEvent::Navigate("goto".to_string(), tab));
            }
            NavigationAction::GoBack => {
                let _ = self.event_tx.send(crate::app::AppEvent::Navigate(
                    "back".to_string(),
                    TabType::Home,
                ));
            }
            NavigationAction::GoToTabWithData(tab, _) => {
                let _ = self
                    .event_tx
                    .send(crate::app::AppEvent::Navigate("goto".to_string(), tab));
            }
            NavigationAction::GoToTabWithQuery(tab, _) => {
                let _ = self
                    .event_tx
                    .send(crate::app::AppEvent::Navigate("goto".to_string(), tab));
            }
        }
    }
}

/// Convenience macros for common navigation patterns
#[macro_export]
macro_rules! nav_to {
    ($navigator:expr, Jobs) => {
        $navigator.to_jobs(None)
    };
    ($navigator:expr, Jobs, $job_id:expr) => {
        $navigator.to_jobs(Some($job_id.to_string()))
    };
    ($navigator:expr, Experiments) => {
        $navigator.to_experiments(None)
    };
    ($navigator:expr, Experiments, $exp_id:expr) => {
        $navigator.to_experiments(Some($exp_id.to_string()))
    };
    ($navigator:expr, Compute) => {
        $navigator.to_compute()
    };
    ($navigator:expr, Home) => {
        $navigator.to_home()
    };
    ($navigator:expr, Back) => {
        $navigator.go_back()
    };
}
