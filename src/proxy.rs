use indicatif::{ProgressBar, ProgressStyle};

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Cursor, path::Path, time::Duration};
use tokio::task::JoinHandle;

use anyhow::{Context, Result};

use clap::{Parser, Subcommand};

pub struct Install {
    pub version: String,
    pub path: PathBuf,
}

fn list_local_installs() -> Result<Vec<PathBuf>> {
    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let install_dir = tamago_dir.join("install");

    let mut versions = vec![];

    for entry in std::fs::read_dir(install_dir)? {
        let entry = entry?;
        let path = entry.path();
        versions.push(path);
    }

    Ok(versions)
}

fn list_latest_installs() -> Result<Vec<Install>> {
    let installs = list_local_installs()?;

    let mut latest_installs = vec![];
    for install in &installs {
        let version = install.file_name().unwrap().to_str().unwrap().to_string();
        let install = Install {
            version: version.clone(),
            path: install.clone(),
        };
        latest_installs.push(install);
    }

    // TODO: Filter out any old patch versions.

    Ok(latest_installs)
}

pub fn find_install(minor_version: &str) -> Result<Install> {
    // Find and return the matching install.
    let installs = list_latest_installs()?;
    for install in installs {
        if install.version.starts_with(minor_version) {
            return Ok(install);
        }
    }

    anyhow::bail!("failed to find install for {}", minor_version);
}

pub fn proxy_python(install_dir: &Path, args: &[&str]) -> Result<()> {
    let python_path = install_dir.join("bin").join("python3");

    let mut child = Command::new(python_path)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let exit_status = child.wait()?;
    std::process::exit(exit_status.code().unwrap_or(1));
}
