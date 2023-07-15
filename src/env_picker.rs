use std::path::Path;

use anyhow::Result;

use crate::proxy::Install;

fn parse_python_version(version: &str) -> Result<(u32, u32, u32)> {
    let mut version = version.trim().split(".");
    let major = version.next().unwrap().parse::<u32>()?;
    let minor = version.next().unwrap().parse::<u32>()?;
    let patch = version.next().unwrap().parse::<u32>()?;
    Ok((major, minor, patch))
}

fn parse_python_version_file(version_file: &Path) -> Result<(u32, u32, u32)> {
    let version = std::fs::read_to_string(version_file)?;
    parse_python_version(&version)
}

pub fn find_project_python_version() -> Result<Install> {
    let project_dir = std::env::current_dir()?;
    let version_file = project_dir.join(".python-version");

    let triplet = parse_python_version_file(&version_file)?;

    crate::proxy::find_install(&format!("{}.{}", triplet.0, triplet.1))
}
