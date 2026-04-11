use anyhow::Result;
use clap::Parser;
use fora_tui::config::AppConfig;
use tracing_subscriber::EnvFilter;

mod cli;
mod code;
mod env;
mod init;
mod inputs;
mod settings;
mod submit;
mod utils;
use cli::{Cli, Command};
use submit::submit_to_azure;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Cli::parse();

    match args.command {
        Some(Command::Submit(args)) => {
            submit_to_azure(&args).await?;
        }
        Some(Command::Init) => {
            init::run_init_wizard().await?;
        }
        None => {
            // Check if config exists and has workspaces; if not, run init wizard first
            let config = AppConfig::load()?;
            if config.workspaces.is_empty() {
                init::run_init_wizard().await?;
            }
            fora_tui::run().await?;
        }
    }
    Ok(())
}
