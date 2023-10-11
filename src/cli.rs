use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
pub struct HankArgs {
    /// Load configuration from a custom location. Defaults to: $XDG_CONFIG/hank/config.yml
    #[arg(short, long = "config", value_name = "FILE")]
    pub config_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Print a config template
    ConfigTemplate,
    /// Create a config file. Defaults to: $XDG_CONFIG/hank/config.yml
    Init {
        /// Create configuration at a custom location.
        #[arg(short, long = "config", value_name = "FILE")]
        config_path: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, Parser)]
#[command(subcommand_negates_reqs(true))]
#[command(args_conflicts_with_subcommands(true))]
pub struct Cli {
    #[command(flatten)]
    pub args: HankArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
