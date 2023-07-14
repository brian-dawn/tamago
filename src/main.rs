use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

use std::os::linux::raw;
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
            let home_dir = home::home_dir().context("failed to find home dir")?;
            let install_dir = home_dir.join(".tamago").join("install").join(&version);

            let args: Vec<&str> = raw_remaining_args.iter().map(AsRef::as_ref).collect();

            proxy::proxy_python(&install_dir, &args)?;

            return Ok(());
        }
        None => {}
    }

    let version = "3.11.3";
    // download_and_build(version).await?;

    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let sources_dir = tamago_dir.join("sources");

    let build_dir = tamago_dir.join("build");
    let install_dir = tamago_dir.join("install").join(version);

    // Assume that python_path is a string containing the path to the desired Python interpreter,
    // and that args is a Vec<String> containing the arguments passed by the user.

    // Exit with the same code as the Python process.

    Ok(())
}
