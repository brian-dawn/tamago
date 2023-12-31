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
    /// Run a python binary for a specific version.
    Run {
        /// Python version to use, e.g. 3.11
        version: String,

        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        raw_remaining_args: Vec<String>,
    },

    /// Sets a default python version to use, e.g. 3.11
    Default { version: String },

    /// Returns the python install path based on the current project.
    Find,

    /// Returns the path of the tamago activate shell script.
    Activate,

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

        Some(Commands::Default { version }) => {
            // Make sure we can find it first.
            let (major, minor) = env_picker::parse_python_version(&version)?;
            let found_install = proxy::find_install(&format!("{}.{}", major, minor))?;

            let home_dir = home::home_dir().context("failed to find home dir")?;
            let tamago_dir = home_dir.join(".tamago");

            let default_path = tamago_dir.join("default");
            let default_contents = format!("{}.{}", major, minor);
            std::fs::write(&default_path, default_contents)?;

            println!(
                "setting default python version to {}.{} at {}",
                major,
                minor,
                found_install.path.display()
            );

            return Ok(());
        }

        Some(Commands::Find) => {
            let install = env_picker::find_project_python_version()?;

            print!("{}", install.path.display());

            return Ok(());
        }

        Some(Commands::Activate) => {
            // Make the tamago directory if it doesn't exist.
            let home_dir = home::home_dir().context("failed to find home dir")?;
            let tamago_dir = home_dir.join(".tamago");

            if !tamago_dir.exists() {
                std::fs::create_dir(&tamago_dir).context("failed to create tamago dir")?;
            }

            let activate_path = tamago_dir.join("activate");
            let contents = include_str!("../activate.sh");
            if !activate_path.exists() {
                std::fs::write(&activate_path, contents)
                    .context("failed to write activate file")?;
            }

            print!("{}", activate_path.display());
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
