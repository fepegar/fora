use crate::{
    app::{AppEvent, TabType},
    azure::AzureClient,
    cache::CacheManager,
    navigation::TabNavigator,
    tabs::*,
};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

/// Configuration for creating a tab
#[derive(Clone)]
pub struct TabConfig {
    pub tab_type: TabType,
    pub title: String,
    pub shortcut_key: Option<char>,
    pub icon: Option<&'static str>,
    pub description: Option<String>,
}

/// Factory trait for creating tabs
pub trait TabFactory: Send + Sync {
    fn create_tab(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Box<dyn Tab>;

    fn config(&self) -> TabConfig;
}

/// Home tab factory
pub struct HomeTabFactory;

impl TabFactory for HomeTabFactory {
    fn create_tab(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Box<dyn Tab> {
        Box::new(home::HomeTab::new(azure_client, cache, event_tx))
    }

    fn config(&self) -> TabConfig {
        TabConfig {
            tab_type: TabType::Home,
            title: "Home".to_string(),
            shortcut_key: Some('h'),
            icon: Some("🏠"),
            description: Some("Dashboard and overview".to_string()),
        }
    }
}

/// Jobs tab factory
pub struct JobsTabFactory;

impl TabFactory for JobsTabFactory {
    fn create_tab(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Box<dyn Tab> {
        Box::new(jobs::JobsTab::new(azure_client, cache, event_tx))
    }

    fn config(&self) -> TabConfig {
        TabConfig {
            tab_type: TabType::Jobs,
            title: "Jobs".to_string(),
            shortcut_key: Some('j'),
            icon: Some("⚙️"),
            description: Some("Manage and monitor ML jobs".to_string()),
        }
    }
}

/// Experiments tab factory
pub struct ExperimentsTabFactory;

impl TabFactory for ExperimentsTabFactory {
    fn create_tab(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Box<dyn Tab> {
        Box::new(experiments::ExperimentsTab::new(
            azure_client,
            cache,
            event_tx,
        ))
    }

    fn config(&self) -> TabConfig {
        TabConfig {
            tab_type: TabType::Experiments,
            title: "Experiments".to_string(),
            shortcut_key: Some('e'),
            icon: Some("🧪"),
            description: Some("Track and compare experiments".to_string()),
        }
    }
}

/// Compute tab factory
pub struct ComputeTabFactory;

impl TabFactory for ComputeTabFactory {
    fn create_tab(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Box<dyn Tab> {
        Box::new(compute::ComputeTab::new(azure_client, cache, event_tx))
    }

    fn config(&self) -> TabConfig {
        TabConfig {
            tab_type: TabType::Compute,
            title: "Compute".to_string(),
            shortcut_key: Some('c'),
            icon: Some("💻"),
            description: Some("Manage compute resources".to_string()),
        }
    }
}

/// Central registry for managing all tabs
pub struct TabRegistry {
    factories: HashMap<TabType, Box<dyn TabFactory>>,
    tab_order: Vec<TabType>,
    shortcut_map: HashMap<char, TabType>,
}

impl Default for TabRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TabRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
            tab_order: Vec::new(),
            shortcut_map: HashMap::new(),
        };

        // Register built-in tabs
        registry.register(Box::new(HomeTabFactory));
        registry.register(Box::new(JobsTabFactory));
        registry.register(Box::new(ExperimentsTabFactory));
        registry.register(Box::new(ComputeTabFactory));

        registry
    }

    /// Register a new tab factory
    pub fn register(&mut self, factory: Box<dyn TabFactory>) {
        let config = factory.config();
        let tab_type = config.tab_type.clone();

        // Add to order if not already present
        if !self.tab_order.contains(&tab_type) {
            self.tab_order.push(tab_type.clone());
        }

        // Map shortcut key if provided
        if let Some(key) = config.shortcut_key {
            self.shortcut_map.insert(key, tab_type.clone());
        }

        self.factories.insert(tab_type, factory);
    }

    /// Create a tab instance
    pub fn create_tab(
        &self,
        tab_type: &TabType,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Option<Box<dyn Tab>> {
        self.factories
            .get(tab_type)
            .map(|factory| factory.create_tab(azure_client, cache, event_tx))
    }

    /// Create all tabs
    pub fn create_all_tabs(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> HashMap<TabType, Box<dyn Tab>> {
        let mut tabs = HashMap::new();
        let navigator = TabNavigator::new(event_tx.clone());

        for tab_type in &self.tab_order {
            if let Some(mut tab) = self.create_tab(
                tab_type,
                azure_client.clone(),
                cache.clone(),
                event_tx.clone(),
            ) {
                tab.set_navigator(navigator.clone());
                tabs.insert(tab_type.clone(), tab);
            }
        }

        tabs
    }

    /// Get tab configuration
    pub fn get_config(&self, tab_type: &TabType) -> Option<TabConfig> {
        self.factories.get(tab_type).map(|factory| factory.config())
    }

    /// Get all tab configurations in order
    pub fn get_all_configs(&self) -> Vec<TabConfig> {
        self.tab_order
            .iter()
            .filter_map(|tab_type| self.get_config(tab_type))
            .collect()
    }

    /// Get tab order
    pub fn tab_order(&self) -> &[TabType] {
        &self.tab_order
    }

    /// Get tab type by shortcut key
    pub fn tab_by_shortcut(&self, key: char) -> Option<&TabType> {
        self.shortcut_map.get(&key)
    }

    /// Set custom tab order
    pub fn set_tab_order(&mut self, order: Vec<TabType>) {
        // Only include tabs that are actually registered
        self.tab_order = order
            .into_iter()
            .filter(|tab_type| self.factories.contains_key(tab_type))
            .collect();
    }

    /// Remove a tab from the registry
    pub fn unregister(&mut self, tab_type: &TabType) {
        if let Some(factory) = self.factories.remove(tab_type) {
            // Remove from order
            self.tab_order.retain(|t| t != tab_type);

            // Remove shortcut mapping
            let config = factory.config();
            if let Some(key) = config.shortcut_key {
                if self.shortcut_map.get(&key) == Some(tab_type) {
                    self.shortcut_map.remove(&key);
                }
            }
        }
    }

    /// Check if a tab type is registered
    pub fn is_registered(&self, tab_type: &TabType) -> bool {
        self.factories.contains_key(tab_type)
    }

    /// Get the number of registered tabs
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }

    /// Get all registered tab types
    pub fn registered_types(&self) -> Vec<TabType> {
        self.factories.keys().cloned().collect()
    }
}

/// Builder for creating custom tab registries
pub struct TabRegistryBuilder {
    registry: TabRegistry,
}

impl Default for TabRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TabRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: TabRegistry {
                factories: HashMap::new(),
                tab_order: Vec::new(),
                shortcut_map: HashMap::new(),
            },
        }
    }

    /// Add a tab factory to the builder
    pub fn with_tab(mut self, factory: Box<dyn TabFactory>) -> Self {
        self.registry.register(factory);
        self
    }

    /// Set the tab order
    pub fn with_order(mut self, order: Vec<TabType>) -> Self {
        self.registry.set_tab_order(order);
        self
    }

    /// Build the registry
    pub fn build(self) -> TabRegistry {
        self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::TabType;

    #[test]
    fn test_registry_creation() {
        let registry = TabRegistry::new();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 4); // Home, Jobs, Experiments, Compute
    }

    #[test]
    fn test_shortcut_mapping() {
        let registry = TabRegistry::new();

        assert_eq!(registry.tab_by_shortcut('h'), Some(&TabType::Home));
        assert_eq!(registry.tab_by_shortcut('j'), Some(&TabType::Jobs));
        assert_eq!(registry.tab_by_shortcut('e'), Some(&TabType::Experiments));
        assert_eq!(registry.tab_by_shortcut('c'), Some(&TabType::Compute));
        assert_eq!(registry.tab_by_shortcut('x'), None);
    }

    #[test]
    fn test_tab_order() {
        let registry = TabRegistry::new();
        let order = registry.tab_order();

        assert_eq!(order.len(), 4);
        assert!(order.contains(&TabType::Home));
        assert!(order.contains(&TabType::Jobs));
        assert!(order.contains(&TabType::Experiments));
        assert!(order.contains(&TabType::Compute));
    }

    #[test]
    fn test_builder_pattern() {
        let registry = TabRegistryBuilder::new()
            .with_tab(Box::new(HomeTabFactory))
            .with_tab(Box::new(JobsTabFactory))
            .with_order(vec![TabType::Jobs, TabType::Home])
            .build();

        assert_eq!(registry.len(), 2);
        assert_eq!(registry.tab_order(), &[TabType::Jobs, TabType::Home]);
    }

    #[test]
    fn test_unregister() {
        let mut registry = TabRegistry::new();
        assert!(registry.is_registered(&TabType::Home));

        registry.unregister(&TabType::Home);
        assert!(!registry.is_registered(&TabType::Home));
        assert_eq!(registry.tab_by_shortcut('h'), None);
    }
}
