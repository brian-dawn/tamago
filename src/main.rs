use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

use std::os::linux::raw;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{io::Cursor, path::Path, time::Duration};
use tokio::task::JoinHandle;

use anyhow::{Context, Result};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

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
    Build {
        /// python version to build, e.g. 3.11.3
        version: String,
    },
}

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
    let sources_url = format!("https://www.python.org/ftp/python/{version}/Python-{version}.tgz");

    let home_dir = home::home_dir().context("failed to find home dir")?;
    let tamago_dir = home_dir.join(".tamago");
    let sources_dir = tamago_dir.join("sources");

    let build_dir = tamago_dir.join("build");
    let install_dir = tamago_dir.join("install").join(version);
    let destination = sources_dir.join(sources_url.split("/").last().context("invalid url")?);

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

    // Clear out the sources and the build directory
    std::fs::remove_dir_all(&sources_dir)?;
    std::fs::remove_dir_all(&build_dir)?;

    let _ = spinner_task.abort();

    Ok(())
}

fn proxy_python(install_dir: &Path, args: &[&str]) -> Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build { version }) => {
            download_and_build(&version).await?;
            return Ok(());
        }

        Some(Commands::Run {
            version,
            raw_remaining_args,
        }) => {
            let home_dir = home::home_dir().context("failed to find home dir")?;
            let install_dir = home_dir.join(".tamago").join("install").join(&version);

            let args: Vec<&str> = raw_remaining_args.iter().map(AsRef::as_ref).collect();

            proxy_python(&install_dir, &args)?;

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
