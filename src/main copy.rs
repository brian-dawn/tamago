use std::{io::Cursor, path::Path};
use tokio::fs::File;
use tokio::io::BufReader;

use serde::Deserialize;

use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

async fn find_latest_releases(os: &str, arch: &str) -> Result<Vec<String>> {
    let url = "https://api.github.com/repos/indygreg/python-build-standalone/releases/latest";

    // Create a client
    let client = reqwest::Client::builder()
        .user_agent("tamago/0.1")
        .build()?;

    let latest_release = client.get(url).send().await?.json::<Release>().await?;

    // We want to ignore the v_4 builds since they are very CPU specific.
    // v_2 and v_3 might be what we want but for now we focus on the baseline
    // which provide maximum compatibility.

    let mut urls = Vec::new();
    for asset in latest_release.assets {
        if asset.name.contains(os)
            && asset.name.contains(arch)
            && asset.name.ends_with(".tar.zst")
            && asset.name.contains("musl")
            && asset.name.contains("lto-full")
            && !asset.name.contains("v4")
            && !asset.name.contains("v3")
            && !asset.name.contains("v2")
        {
            //println!("{:?}: {:?}", asset.name, asset.browser_download_url);
            urls.push(asset.browser_download_url.to_string());
        }
        //println!("{:?}: {:?}", asset.name, asset.browser_download_url);
    }
    Ok(urls)
}

async fn fetch_url(url: &str, file_name: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

async fn download() -> Result<()> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    println!("arch: {}", arch);
    println!("os: {}", os);

    // Create the $HOME/.tamago directory if it doesn't exist
    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let downloads_dir = tamago_dir.join("downloads");
    std::fs::create_dir_all(&downloads_dir)?;

    let urls = find_latest_releases(os, arch).await?;

    for url in urls {
        println!("Downloading: {}", url);
        let file_name = url.split("/").last().context("invalid url")?.to_string();
        let path = downloads_dir.join(&file_name);
        fetch_url(&url, &path).await?;
    }

    Ok(())
}

async fn extract() -> Result<()> {
    // List the files in ~/.tamago/downloads
    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let downloads_dir = tamago_dir.join("downloads");

    let installs_dir = tamago_dir.join("installs");

    let mut files = Vec::new();
    for entry in std::fs::read_dir(&downloads_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    // Extract the files
    for file in files {
        let decompressor = decompress::Decompress::default();

        // fnames are of the format cpython-3.8.16...
        // Extract the version number using the regex crate
        let fname = file
            .file_name()
            .context("invalid path")?
            .to_str()
            .context("invalid file name")?;
        let re = regex::Regex::new(r"cpython-(\d+\.\d+\.\d+)").context("invalid regex")?;
        let captures = re.captures(fname).context("invalid captures")?;
        let version = captures.get(1).context("invalid version")?.as_str();

        let version_no_patch = version.split(".").take(2).collect::<Vec<&str>>().join(".");

        let version_dir = installs_dir.join(&version_no_patch);
        std::fs::create_dir_all(&version_dir)?;

        decompressor.decompress(
            file,
            version_dir.clone(),
            &decompress::ExtractOpts { strip: 1 },
        )?;

        // Folder structure is like this:
        // installs/3.8/install/bin/python3.8
        // Echo the path for all these folders...
        let interpreter_path = version_dir
            .join("install")
            .join("bin")
            .join(format!("python{}", &version_no_patch));

        // Make a symlink in ~/.tamago/bin
        let bin_dir = tamago_dir.join("bin");
        let symlink_path = bin_dir.join(format!("python{}", &version_no_patch));

        // Create the bin dir if it doesn't exist
        std::fs::create_dir_all(&bin_dir)?;

        // Make the symlink
        std::os::unix::fs::symlink(&interpreter_path, &symlink_path)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    extract().await?;
    Ok(())
}
