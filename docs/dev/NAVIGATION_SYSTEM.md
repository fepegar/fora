# Scalable Navigation System

This document describes the new scalable navigation system that replaces the previous approach where each tab had hardcoded knowledge of other tabs.

## Problems with the Old System

The original navigation system had several scalability issues:

1. **Tight coupling**: Each tab had explicit knowledge of other tab types and their specific events
2. **Hardcoded dependencies**: Adding a new tab required updating multiple files
3. **Scattered navigation logic**: Navigation code was duplicated across tabs
4. **Event proliferation**: Each tab needed its own event types in the main `AppEvent` enum

## New Architecture

### 1. Centralized Navigation Manager

The `NavigationManager` handles all navigation state and history:

```rust
pub struct NavigationManager {
    event_tx: UnboundedSender<AppEvent>,
    context: NavigationContext,
    history: Vec<TabType>,
    custom_data_store: HashMap<String, String>,
}
```

### 2. Tab Navigator

Each tab receives a `TabNavigator` that provides simple navigation methods:

```rust
pub struct TabNavigator {
    event_tx: UnboundedSender<AppEvent>,
}

impl TabNavigator {
    pub fn to_jobs(&self, job_id: Option<String>) { /* ... */ }
    pub fn to_experiments(&self, experiment_id: Option<String>) { /* ... */ }
    pub fn to_compute(&self) { /* ... */ }
    pub fn to_home(&self) { /* ... */ }
    pub fn go_back(&self) { /* ... */ }
}
```

### 3. Tab Registry System

The `TabRegistry` manages all available tabs using a factory pattern:

```rust
pub struct TabRegistry {
    factories: HashMap<TabType, Box<dyn TabFactory>>,
    tab_order: Vec<TabType>,
    shortcut_map: HashMap<char, TabType>,
}
```

### 4. Updated Tab Trait

The `Tab` trait now includes navigation support:

```rust
#[async_trait]
pub trait Tab: Send + Sync {
    async fn initialize(&mut self);
    async fn refresh(&mut self);
    async fn handle_key(&mut self, key: KeyEvent);
    fn render(&self, f: &mut Frame, area: Rect);
    fn title(&self) -> &str;

    // Navigation support
    async fn on_navigation(&mut self, context: &NavigationContext);
    fn set_navigator(&mut self, navigator: TabNavigator);

    // For downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

## How to Use

### Adding a New Tab

1. **Create the tab struct** implementing the `Tab` trait:

```rust
pub struct MyNewTab {
    navigator: Option<TabNavigator>,
    // ... other fields
}

#[async_trait]
impl Tab for MyNewTab {
    // ... implement all required methods
    
    fn set_navigator(&mut self, navigator: TabNavigator) {
        self.navigator = Some(navigator);
    }
    
    async fn on_navigation(&mut self, context: &NavigationContext) {
        // Handle navigation context (selection, query params, etc.)
    }
}
```

2. **Create a factory** for the tab:

```rust
pub struct MyNewTabFactory;

impl TabFactory for MyNewTabFactory {
    fn create_tab(
        &self,
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Box<dyn Tab> {
        Box::new(MyNewTab::new(azure_client, cache, event_tx))
    }

    fn config(&self) -> TabConfig {
        TabConfig {
            tab_type: TabType::MyNew,
            title: "My New Tab".to_string(),
            shortcut_key: Some('n'),
            icon: Some("📝"),
            description: Some("My new tab description".to_string()),
        }
    }
}
```

3. **Register the tab** in the registry:

```rust
let mut registry = TabRegistry::new();
registry.register(Box::new(MyNewTabFactory));
```

### Navigation in Tabs

Use the provided navigator or convenience macros:

```rust
// Using the navigator directly
if let Some(navigator) = &self.navigator {
    navigator.to_jobs(Some("job-123".to_string()));
    navigator.to_experiments(None);
    navigator.go_back();
}

// Using convenience macros
nav_to!(navigator, Jobs);
nav_to!(navigator, Jobs, "job-123");
nav_to!(navigator, Experiments);
nav_to!(navigator, Back);
```

### Handling Navigation Context

Tabs can respond to navigation context:

```rust
async fn on_navigation(&mut self, context: &NavigationContext) {
    // Check if we came from another tab
    if let Some(previous_tab) = &context.previous_tab {
        println!("Came from: {:?}", previous_tab);
    }
    
    // Check for selection data
    if let Some(selection_id) = &context.selection_id {
        self.select_item(selection_id);
    }
    
    // Check query parameters
    for (key, value) in &context.query_params {
        self.handle_query_param(key, value);
    }
}
```

## Benefits

1. **Loose coupling**: Tabs only depend on the `TabNavigator` interface
2. **Easy extensibility**: Adding new tabs requires minimal changes
3. **Centralized navigation**: All navigation logic is in one place
4. **Consistent API**: All tabs use the same navigation methods
5. **History support**: Built-in back/forward navigation
6. **Context passing**: Support for passing data between tabs
7. **Keyboard shortcuts**: Automatic shortcut registration and handling

## Migration Guide

To migrate an existing tab:

1. Add `navigator: Option<TabNavigator>` field
2. Implement `set_navigator()` and `on_navigation()` methods
3. Replace hardcoded navigation with navigator calls
4. Remove tab-specific navigation events from `AppEvent` enum
5. Create a factory for the tab and register it

## Example: Before and After

### Before (Old System)
```rust
// In HomeTab
HomeEvent::NavigateToJob(job_id) => {
    let _ = self.event_tx.send(AppEvent::TabChanged(TabType::Jobs));
    let _ = self.event_tx.send(AppEvent::JobsEvent(
        JobsEvent::JobSelected(job_id),
    ));
}
```

### After (New System)
```rust
// In HomeTab
if let Some(navigator) = &self.navigator {
    nav_to!(navigator, Jobs, job_id);
}
```

The new system is more concise, type-safe, and doesn't require knowledge of other tabs' internal events.