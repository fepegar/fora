use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;
mod code;
mod env;
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
        None => {
            azure_tui::run().await?;
        }
    }
    Ok(())
}
