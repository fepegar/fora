pub mod app;
pub mod cache;
pub mod client;
pub mod components;
pub mod config;
pub mod event;
pub mod tabs;
pub mod theme;
pub mod widgets;

use anyhow::Result;
use config::AppConfig;

/// Entry point for the TUI application.
pub async fn run() -> Result<()> {
    let config = AppConfig::load()?;
    let mut app = app::App::new(config).await?;
    app.run().await
}
