use indicatif::{ProgressBar, ProgressStyle};

use std::os::linux::raw;
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

pub fn list_latest_installs() -> Result<Vec<Install>> {
    let installs = list_local_installs()?;

    // For each version, find the latest patch.
    let mut versions = vec![];

    for install in installs {
        let mut version = install.file_name().unwrap().to_str().unwrap().to_string();

        let mut version_parts = version.splitn(2, ".");

        let major = version_parts.next().unwrap();
        let minor = version_parts.next().unwrap();
        let patch = version_parts.next().unwrap();

        let mut patch_parts = patch.splitn(2, "-");

        let patch = patch_parts.next().unwrap();
        let patch = patch.parse::<u32>().unwrap();

        let patch = patch_parts.next().unwrap();
        let patch = patch.parse::<u32>().unwrap();

        version = format!("{}.{}.{}", major, minor, patch);

        versions.push(Install {
            version,
            path: install,
        });
    }

    Ok(versions)
}




pub fn proxy_python(install_dir: &Path, args: &[&str]) -> Result<()> {
    let python_path = install_dir.join("bin").join("python3");
    println!("{}", python_path.display());
    // let output = Command::new(python_path)
    //     // .args(&args)
    //     .output()
    //     .expect("Failed to execute command");

    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    let mut child = Command::new(python_path)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let exit_status = child.wait()?;
    std::process::exit(exit_status.code().unwrap_or(1));
}
