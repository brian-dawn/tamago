use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Cursor, path::Path, time::Duration};
use tokio::task::JoinHandle;

use anyhow::{Context, Result};

const PYTHON_VERSIONS: [&str; 4] = ["3.8", "3.9", "3.10", "3.11"];

pub async fn fetch_python_versions() -> Result<HashSet<String>> {
    let mut versions = HashSet::new();

    let response = reqwest::get("https://www.python.org/downloads/source/").await?;
    let body = response.text().await?;

    // Just use regex to find all Python-3.x.x strings, note we can ignore alpha releases by making sure x is only digits.
    let re = Regex::new(r"Python-3\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(&body) {
        versions.insert(cap[0].to_string().replace("Python-", ""));
    }

    Ok(versions)
}

pub fn find_latest_patches(versions: &HashSet<String>) -> Result<Vec<String>> {
    let mut patches = Vec::new();

    for version in PYTHON_VERSIONS.iter() {
        let mut version_patches = Vec::new();
        for patch in versions.iter() {
            if patch.starts_with(version) {
                version_patches.push(patch.clone());
            }
        }

        if version_patches.is_empty() {
            continue;
        }

        // Find the latest patch version, sort by the patch number.
        version_patches.sort_by(|a, b| {
            let a_patch = a.split(".").last().unwrap().parse::<u32>().unwrap();
            let b_patch = b.split(".").last().unwrap().parse::<u32>().unwrap();
            a_patch.cmp(&b_patch)
        });

        // Add the latest patch to the list of patches.
        patches.push(version_patches.pop().context("failed to find a version!")?);
    }

    Ok(patches)
}

async fn fetch_url(url: &str, file_name: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

pub async fn download_and_build_all() -> Result<()> {
    let versions = fetch_python_versions().await?;
    let patches = find_latest_patches(&versions)?;
    for patch in patches.iter() {
        download_and_build(patch).await?;
    }

    Ok(())
}

pub async fn download_and_build(version: &str) -> Result<()> {
    let sources_url = format!("https://www.python.org/ftp/python/{version}/Python-{version}.tgz");

    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let sources_dir = tamago_dir.join("sources");

    let build_dir = tamago_dir.join("build");
    let install_dir = tamago_dir.join("install").join(version);
    let destination = sources_dir.join(sources_url.split("/").last().context("invalid url")?);

    let mut template = String::new();
    template.push_str("{spinner} ");
    template.push_str("Building Python ");
    template.push_str(version);
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("/|\\- ")
        .template(&template)?;
    let spinner = ProgressBar::new_spinner().with_style(spinner_style);

    // Make all the directories if they don't exist.
    std::fs::create_dir_all(&sources_dir)?;
    std::fs::create_dir_all(&build_dir)?;

    fetch_url(&sources_url, &destination).await?;

    let decompressor = decompress::Decompress::default();
    decompressor.decompress(
        destination,
        build_dir.clone(),
        &decompress::ExtractOpts { strip: 1 },
    )?;

    // Create a spinner task
    let spinner_task: JoinHandle<()> = tokio::spawn(async move {
        while !spinner.is_finished() {
            spinner.tick();
            tokio::time::sleep(Duration::from_millis(100)).await; // adjust speed here
        }
    });

    let mut configure = tokio::process::Command::new("./configure")
        // Disabled for now to increase build speed.
        // .arg("--enable-optimizations")
        .arg(format!("--prefix={}", install_dir.display()))
        .current_dir(&build_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to run configure")?;

    configure.wait().await?;

    let default_parallelism_approx = std::thread::available_parallelism()?.get();

    let mut make = tokio::process::Command::new("make")
        .arg("-j")
        .arg(format!("{}", default_parallelism_approx))
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

    // Clear out the sources and the build directory
    std::fs::remove_dir_all(&sources_dir)?;
    std::fs::remove_dir_all(&build_dir)?;

    let _ = spinner_task.abort();

    Ok(())
}
