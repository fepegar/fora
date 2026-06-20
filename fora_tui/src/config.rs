use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::components::files_view::FilesConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default, skip_serializing_if = "UiConfig::is_default")]
    pub ui: UiConfig,
    /// Name of the workspace to show on TUI launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace: Option<String>,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceConfig>,
    #[serde(default, skip_serializing_if = "ColumnsConfig::is_empty")]
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
    /// IANA timezone name (e.g. "Europe/London"). Defaults to UTC if unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Polling interval for live-tailing the previewed run file (seconds).
    ///
    /// When the previewed file belongs to an active run, the Files sub-tab
    /// HEADs the underlying blob every `file_preview_refresh_secs` seconds
    /// and refetches the suffix on growth. Set `0` to disable tailing.
    #[serde(default = "default_file_preview_refresh")]
    pub file_preview_refresh_secs: u64,
    /// syntect theme name used for syntax-highlighted previews.
    ///
    /// Defaults to `base16-ocean.dark` (bundled with syntect). Any theme
    /// name from `syntect::highlighting::ThemeSet::load_defaults()` works.
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
    /// Directory where the `s` hotkey writes saved files.
    ///
    /// Defaults to the current working directory (`./`). Supports `~`
    /// expansion.
    #[serde(default = "default_save_dir")]
    pub save_dir: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_help_bar: default_show_help_bar(),
            refresh_interval_secs: default_refresh_interval(),
            timezone: None,
            file_preview_refresh_secs: default_file_preview_refresh(),
            syntax_theme: default_syntax_theme(),
            save_dir: default_save_dir(),
        }
    }
}

impl UiConfig {
    fn is_default(&self) -> bool {
        self.show_help_bar == default_show_help_bar()
            && self.refresh_interval_secs == default_refresh_interval()
            && self.timezone.is_none()
            && self.file_preview_refresh_secs == default_file_preview_refresh()
            && self.syntax_theme == default_syntax_theme()
            && self.save_dir == default_save_dir()
    }

    /// Returns the configured timezone, falling back to UTC if unset or invalid.
    pub fn tz(&self) -> chrono_tz::Tz {
        self.timezone
            .as_deref()
            .and_then(|s| s.parse::<chrono_tz::Tz>().ok())
            .unwrap_or(chrono_tz::Tz::UTC)
    }
}

impl UiConfig {
    /// Build the FilesView's tunables from the UI config.
    pub fn files_config(&self) -> FilesConfig {
        FilesConfig {
            tail_interval: if self.file_preview_refresh_secs == 0 {
                None
            } else {
                Some(std::time::Duration::from_secs(
                    self.file_preview_refresh_secs,
                ))
            },
            syntax_theme: self.syntax_theme.clone(),
            save_dir: self.save_dir.clone(),
        }
    }
}

fn default_show_help_bar() -> bool {
    true
}

fn default_refresh_interval() -> u64 {
    30
}

fn default_file_preview_refresh() -> u64 {
    3
}

fn default_syntax_theme() -> String {
    "base16-ocean.dark".to_string()
}

fn default_save_dir() -> String {
    "./".to_string()
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
#[serde(default)]
pub struct ColumnsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute: Option<Vec<String>>,
}

impl ColumnsConfig {
    fn is_empty(&self) -> bool {
        self.jobs.is_none() && self.compute.is_none()
    }
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
            ui: UiConfig::default(),
            default_workspace: None,
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
