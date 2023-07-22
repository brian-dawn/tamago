use std::path::Path;

use anyhow::{Context, Result};
use semver::{BuildMetadata, Prerelease, Version};

use crate::proxy::Install;

#[derive(serde::Deserialize)]
struct Depdendencies {
    python: Option<String>,
}

#[derive(serde::Deserialize)]
struct PyPoetryToml {
    dependencies: Option<Depdendencies>,
}

#[derive(serde::Deserialize)]

struct Tool {
    poetry: Option<PyPoetryToml>,
}

#[derive(serde::Deserialize)]
struct PyProjectToml {
    tool: Tool,
}

fn parse_pyproject(project_dir: &Path) -> Result<PyProjectToml> {
    let pyproject = project_dir.join("pyproject.toml");
    let pyproject = std::fs::read_to_string(pyproject)?;
    let pyproject: PyProjectToml = toml::from_str(&pyproject)?;
    Ok(pyproject)
}

fn choose_install_from_semver(semver: &str) -> Result<Install> {
    let req = semver::VersionReq::parse(semver)?;

    let mut installs = crate::proxy::list_latest_installs()?;

    // Sort by major, minor, and patch in descending order.
    installs.sort_by(|a, b| {
        let a_triplet = parse_python_version_triplet(&a.version).unwrap();
        let b_triplet = parse_python_version_triplet(&b.version).unwrap();

        if a_triplet.0 != b_triplet.0 {
            return b_triplet.0.cmp(&a_triplet.0);
        }

        if a_triplet.1 != b_triplet.1 {
            return b_triplet.1.cmp(&a_triplet.1);
        }

        if a_triplet.2 != b_triplet.2 {
            return b_triplet.2.cmp(&a_triplet.2);
        }

        std::cmp::Ordering::Equal
    });

    for install in installs {
        let triplet = parse_python_version_triplet(&install.version)?;

        let version = Version {
            major: triplet.0,
            minor: triplet.1,
            patch: triplet.2,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        };

        if req.matches(&version) {
            return Ok(install);
        }
    }

    anyhow::bail!("failed to find install for {}", semver);
}

/// Parse a python version string into major and minor version numbers.
pub fn parse_python_version(version: &str) -> Result<(u64, u64)> {
    let mut version = version.trim().split(".");
    let major = version.next().unwrap().parse::<u64>()?;
    let minor = version.next().unwrap().parse::<u64>()?;
    Ok((major, minor))
}

/// Parse a python version string into major and minor version numbers.
fn parse_python_version_triplet(version: &str) -> Result<(u64, u64, u64)> {
    let mut version = version.trim().split(".");
    let major = version.next().unwrap().parse::<u64>()?;
    let minor = version.next().unwrap().parse::<u64>()?;
    let patch = version.next().unwrap().parse::<u64>()?;
    Ok((major, minor, patch))
}

fn parse_python_version_file(version_file: &Path) -> Result<(u64, u64)> {
    // TODO: Look in above directories.

    let version = std::fs::read_to_string(version_file)?;
    parse_python_version(&version)
}

fn load_default_file() -> Result<Install> {
    // Fall back to the default python version if it exists.
    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let default_path = tamago_dir.join("default");

    let (major, minor) = parse_python_version_file(&default_path)?;
    crate::proxy::find_install(&format!("{}.{}", major, minor))
}

fn load_from_pyproject(project_dir: &Path) -> Result<Install> {
    // TODO: Look in above directories.

    // Attempt to load from pyproject.toml.
    let pyproject = parse_pyproject(&project_dir)?;
    let semver = pyproject
        .tool
        .poetry
        .context("no poetry subsection")?
        .dependencies
        .context("no dependencies subsection")?
        .python
        .context("no python field")?;

    choose_install_from_semver(&semver)
}

fn load_from_version_file(project_dir: &Path) -> Result<Install> {
    let version_file = project_dir.join(".python-version");
    let (major, minor) = parse_python_version_file(&version_file)?;
    crate::proxy::find_install(&format!("{}.{}", major, minor))
}

pub fn find_project_python_version() -> Result<Install> {
    let project_dir = std::env::current_dir()?;

    // First attempt to load from version, then pyproject, then default.
    let install = load_from_version_file(&project_dir)
        .or_else(|_| load_from_pyproject(&project_dir))
        .or_else(|_| load_default_file())
        .context("failed to find a python version")?;

    Ok(install)
}
