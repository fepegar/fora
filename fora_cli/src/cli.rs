use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::inputs::InputEnum;
use crate::settings::EnvironmentVariable;

pub const HEADING_UV: &str = "UV Options";
pub const HEADING_ENVIRONMENT: &str = "Environment Options";

#[derive(Parser, Debug)]
#[command(name = "fora", styles = get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Submit(SubmitArgs),
}

#[derive(Args, Debug)]
#[command(about = "Submit a job to Azure ML")]
pub struct SubmitArgs {
    // TODO: Move these out into a separate struct?
    #[arg(long, short, help = "Azure subscription ID")]
    pub subscription: String,

    #[arg(long, short, help = "Azure resource group name")]
    pub resource_group: String,

    #[arg(long, short, help = "Azure ML workspace name")]
    pub workspace: String,

    #[arg(long, short, help = "Name to use for the job")]
    pub name: Option<String>,

    #[arg(long, short, help = "Name of the experiment to submit the job under")]
    pub experiment: String,

    #[arg(
        long,
        help = "Path to the source code to upload. If not specified, the current working directory will be used"
    )]
    pub source: Option<PathBuf>,

    #[arg(
        long,
        help = "Command prefix to prepend to the job command. Useful for setting up environment variables or running under a profiler",
        default_value = "uv run"
    )]
    pub command_prefix: Option<String>,

    #[arg(
        long,
        help = "Executable to use for the job. If not specified, the default is `python`",
        default_value = "python"
    )]
    pub executable: Option<String>,

    #[arg(long, short, help = "Azure ML compute cluster to submit the job to")]
    pub cluster: String,

    #[arg(
        long,
        num_args = 0..,
        help = "Environment variable in the format of `NAME=VALUE`. Can be repeated for multiple environment variables"
    )]
    pub set: Vec<EnvironmentVariable>,

    #[arg(
        long,
        short,
        num_args = 0..,
        help = "Mount input in the format of `alias=data_asset_id:version` (version is optional) or `alias=datastore/path/in/datastore`. Can be repeated for multiple inputs"
    )]
    pub mount: Vec<InputEnum>,

    #[command(flatten)]
    pub env_args: BaseEnvironmentArgs,

    #[command(flatten)]
    pub uv_args: UvArgs,

    #[arg(num_args = 1.., trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum EnvironmentType {
    // #[default]
    // Auto,
    Docker,
    #[default]
    Uv,
}

#[derive(Args, Debug)]
#[command(next_help_heading = HEADING_ENVIRONMENT)]
pub struct BaseEnvironmentArgs {
    #[arg(
        long,
        value_enum,
        default_value_t,
        help = "Type of environment to use for the job."
    )]
    pub env_type: EnvironmentType,

    #[arg(long, help = "Base Docker image to use for the environment.")]
    pub base_docker_image: Option<String>,

    // TODO: Change to Option<bool> when config files are set up
    #[arg(
        long,
        default_value_t = true,
        help = "Whether to automatically rebuild on base image changes."
    )]
    pub auto_rebuild: bool,
}

#[derive(Args, Debug)]
#[command(next_help_heading = HEADING_UV)]
pub struct UvArgs {
    #[arg(
        long,
        help = "Path to the project directory. If not set, defaults to the source directory."
    )]
    pub project_dir: Option<PathBuf>,

    // #[arg(long, default_value_t = false)]
    // pub dont_prebuild_environment: bool,
    //
    #[arg(long)]
    pub uv_extra: Vec<String>,

    #[arg(long)]
    pub uv_group: Vec<String>,
}

pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
}
