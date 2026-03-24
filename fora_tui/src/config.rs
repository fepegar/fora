use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Username for filtering recent jobs (must match the `mlflow.user` tag).
    pub username: String,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceConfig>,
    #[serde(default)]
    pub columns: ColumnsConfig,
    /// Path this config was loaded from (not serialized).
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    #[serde(default = "default_show_help_bar")]
    pub show_help_bar: bool,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_help_bar: default_show_help_bar(),
            refresh_interval_secs: default_refresh_interval(),
        }
    }
}

fn default_show_help_bar() -> bool {
    true
}

fn default_refresh_interval() -> u64 {
    30
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub subscription_id: String,
    pub resource_group: String,
    pub workspace_name: String,
    /// Azure region (e.g. "eastus2") used for the MLflow API endpoint.
    pub region: String,
}

/// Column layout configuration for each tab.
/// Listed columns are visible in that order; unlisted columns are hidden.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ColumnsConfig {
    pub jobs: Option<Vec<String>>,
    pub compute: Option<Vec<String>>,
}

fn config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("fora.toml"))
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // Local config takes priority
        let local_path = PathBuf::from("fora.toml");
        if local_path.exists() {
            let content = std::fs::read_to_string(&local_path)?;
            let mut config: AppConfig = toml::from_str(&content)?;
            config.config_path = Some(local_path);
            return Ok(config);
        }

        // Then check ~/.config/fora.toml
        if let Some(global_path) = config_path() {
            if global_path.exists() {
                let content = std::fs::read_to_string(&global_path)?;
                let mut config: AppConfig = toml::from_str(&content)?;
                config.config_path = Some(global_path);
                return Ok(config);
            }
        }

        // No config file — use defaults with no workspaces
        Ok(AppConfig {
            username: String::new(),
            ui: UiConfig::default(),
            workspaces: Vec::new(),
            columns: ColumnsConfig::default(),
            config_path: None,
        })
    }

    pub fn global_config_path() -> String {
        config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/fora.toml".to_string())
    }

    /// Save column configuration to the config file.
    /// Reads the existing file, updates only the `[columns]` section, and writes back.
    pub fn save_columns(&self) -> anyhow::Result<()> {
        let path = self
            .config_path
            .clone()
            .or_else(config_path)
            .ok_or_else(|| anyhow::anyhow!("No config path available"))?;

        // Read existing file (or start with empty table)
        let existing = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };

        let mut doc: toml::Value =
            toml::from_str(&existing).unwrap_or(toml::Value::Table(toml::map::Map::new()));

        // Serialize just the columns section and merge it in
        let columns_value = toml::Value::try_from(&self.columns)?;
        if let toml::Value::Table(ref mut table) = doc {
            table.insert("columns".to_string(), columns_value);
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let output = toml::to_string_pretty(&doc)?;
        std::fs::write(&path, output)?;

        Ok(())
    }
}
