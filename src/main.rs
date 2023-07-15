use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Cursor, path::Path, time::Duration};
use tokio::task::JoinHandle;

use anyhow::{Context, Result};

use clap::{Parser, Subcommand};

pub mod build_python;
pub mod env_picker;
pub mod proxy;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        /// python version to use, e.g. 3.11.3
        version: String,

        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        raw_remaining_args: Vec<String>,
    },

    /// Returns the python install path based on the current project.
    Find,

    /// build a python version from source
    Build,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build) => {
            build_python::download_and_build_all().await?;

            return Ok(());
        }

        Some(Commands::Find) => {
            let install = env_picker::find_project_python_version()?;

            print!("{}", install.path.display());

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
