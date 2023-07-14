use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Cursor, path::Path, time::Duration};
use tokio::task::JoinHandle;

use anyhow::{Context, Result};

use clap::{Parser, Subcommand};

pub mod build_python;
pub mod proxy;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Run {
        /// python version to use, e.g. 3.11.3
        version: String,

        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        raw_remaining_args: Vec<String>,
    },

    /// build a python version from source
    Build {},
}

// build relevant python versions automatically, allow fetching updates
// act as a proxy for a python version based on the .python-version file or the pyproject.toml file.

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build {}) => {
            build_python::download_and_build_all().await?;

            return Ok(());
        }

        Some(Commands::Run {
            version,
            raw_remaining_args,
        }) => {
            let latest = proxy::find_install(&version)?;

            let args: Vec<&str> = raw_remaining_args.iter().map(AsRef::as_ref).collect();

            proxy::proxy_python(&latest.path, &args)?;

            return Ok(());
        }
        None => {}
    }

    Ok(())
}
