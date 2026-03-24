use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// On-disk cache of experiment id → name mappings, keyed by workspace.
///
/// Stored at `~/.cache/fora/experiments.json`. Each workspace gets its own
/// entry so switching workspaces doesn't lose data.
#[derive(Debug, Default, Serialize, Deserialize)]
struct DiskCache {
    /// Keyed by `"{subscription_id}/{resource_group}/{workspace_name}"`.
    workspaces: HashMap<String, HashMap<String, String>>,
}

/// A handle to the experiment cache for one workspace.
#[derive(Debug, Clone)]
pub struct ExperimentDiskCache {
    workspace_key: String,
    path: PathBuf,
    pub experiments: HashMap<String, String>,
}

impl ExperimentDiskCache {
    /// Load (or create) a cache for the given workspace.
    pub fn load(subscription_id: &str, resource_group: &str, workspace_name: &str) -> Self {
        let workspace_key = format!("{}/{}/{}", subscription_id, resource_group, workspace_name);
        let path = Self::cache_path();

        let experiments = match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<DiskCache>(&contents) {
                Ok(dc) => dc
                    .workspaces
                    .get(&workspace_key)
                    .cloned()
                    .unwrap_or_default(),
                Err(e) => {
                    tracing::debug!("Failed to parse experiment cache: {}", e);
                    HashMap::new()
                }
            },
            Err(_) => HashMap::new(),
        };

        Self {
            workspace_key,
            path,
            experiments,
        }
    }

    /// Merge new experiments into the cache and persist to disk.
    pub fn save(&mut self, experiments: &HashMap<String, String>) {
        self.experiments = experiments.clone();

        // Read existing disk cache to preserve other workspaces' data
        let mut disk_cache = match std::fs::read_to_string(&self.path) {
            Ok(contents) => serde_json::from_str::<DiskCache>(&contents).unwrap_or_default(),
            Err(_) => DiskCache::default(),
        };

        disk_cache
            .workspaces
            .insert(self.workspace_key.clone(), self.experiments.clone());

        if let Some(parent) = self.path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::debug!("Failed to create cache directory: {}", e);
                return;
            }
        }

        match serde_json::to_string_pretty(&disk_cache) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.path, json) {
                    tracing::debug!("Failed to write experiment cache: {}", e);
                }
            }
            Err(e) => {
                tracing::debug!("Failed to serialize experiment cache: {}", e);
            }
        }
    }

    fn cache_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("fora")
            .join("experiments.json")
    }
}
