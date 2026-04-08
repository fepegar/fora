pub mod app;
pub mod cache;
pub mod client;
pub mod components;
pub mod config;
pub mod event;
pub mod experiment_cache;
pub mod format;
pub mod tabs;
pub mod theme;
pub mod widgets;

use std::sync::Arc;

use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::discovery::DiscoveryClient;
use config::AppConfig;

/// Entry point for the TUI application.
pub async fn run() -> Result<()> {
    let config = AppConfig::load()?;
    let username = fetch_display_name().await;
    let mut app = app::App::new(config, username).await?;
    app.run().await
}

/// Fetch the current user's display name from Microsoft Graph.
/// Returns an empty string on failure (graceful degradation).
async fn fetch_display_name() -> String {
    let credential: Arc<dyn azure_core::credentials::TokenCredential> =
        match AzureCliCredential::new(None) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to create Azure credential for Graph API: {}", e);
                return String::new();
            }
        };

    let client = DiscoveryClient::new(credential);
    match client.get_current_user().await {
        Ok(profile) => profile.name(),
        Err(e) => {
            tracing::warn!("Failed to fetch user profile from Graph API: {}", e);
            String::new()
        }
    }
}
