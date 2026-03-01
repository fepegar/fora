use anyhow::Result;
use clap::{ColorChoice, CommandFactory, FromArgMatches, Parser};
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
mod code;
mod inputs;
mod settings;
mod submit;
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
            let mut cmd = Cli::command();
            cmd.print_long_help()?;
        }
    }
    Ok(())
}
