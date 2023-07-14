use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

use std::process::{Command, Stdio};
use std::{io::Cursor, path::Path, time::Duration};
use tokio::task::JoinHandle;

use anyhow::{Context, Result};

// build relevant python versions automatically, allow fetching updates
// act as a proxy for a python version based on the .python-version file or the pyproject.toml file.

async fn fetch_url(url: &str, file_name: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn download_and_build(version: &str) -> Result<()> {
    let wow = format!("https://www.python.org/ftp/python/{version}/Python-{version}.tgz");

    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let sources_dir = tamago_dir.join("sources");

    let build_dir = tamago_dir.join("build");
    let install_dir = tamago_dir.join("install").join(version);
    let destination = sources_dir.join(wow.split("/").last().context("invalid url")?);

    // Make all the directories if they don't exist.
    std::fs::create_dir_all(&sources_dir)?;
    std::fs::create_dir_all(&build_dir)?;

    fetch_url(&wow, &destination).await?;

    let decompressor = decompress::Decompress::default();
    decompressor.decompress(
        destination,
        build_dir.clone(),
        &decompress::ExtractOpts { strip: 1 },
    )?;

    // Run ./configure in the build directory

    // Create a "spinning" style
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("/|\\- ")
        .template("{spinner} Building...")?;

    // Create a spinner with the "spinning" style
    let spinner = ProgressBar::new_spinner().with_style(spinner_style);

    // Create a spinner task
    let spinner_task: JoinHandle<()> = tokio::spawn(async move {
        while !spinner.is_finished() {
            spinner.tick();
            tokio::time::sleep(Duration::from_millis(100)).await; // adjust speed here
        }
    });

    let mut configure = tokio::process::Command::new("./configure")
        .arg("--enable-optimizations")
        .arg(format!("--prefix={}", install_dir.display()))
        .current_dir(&build_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to run configure")?;

    configure.wait().await?;

    let mut make = tokio::process::Command::new("make")
        .arg("-j")
        .arg("8")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .current_dir(&build_dir)
        .spawn()
        .context("failed to run make")?;

    make.wait().await?;

    let mut make_install = tokio::process::Command::new("make")
        .arg("install")
        .current_dir(&build_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to run make install")?;

    make_install.wait().await?;

    let _ = spinner_task.abort();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let version = "3.8.12";
    download_and_build(version).await?;

    Ok(())
}
