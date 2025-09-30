//! Configuration module for AZML TUI application

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub azure: AzureConfig,
    pub ui: UiConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub subscription_id: Option<String>,
    pub resource_group: Option<String>,
    pub workspace_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub refresh_interval_seconds: u64,
    pub max_items_per_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub cache_dir: PathBuf,
    pub ttl_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            azure: AzureConfig::default(),
            ui: UiConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Default for AzureConfig {
    fn default() -> Self {
        Self {
            subscription_id: None,
            resource_group: None,
            workspace_name: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 30,
            max_items_per_page: 50,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| std::env::temp_dir())
                .join("azml-tui"),
            ttl_seconds: 300, // 5 minutes
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Implement configuration loading from file
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement configuration saving to file
        Ok(())
    }
}
