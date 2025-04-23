use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Subcommand, Args, ValueEnum};
use cargo_plumbing::CargoResult;

#[derive(Debug, Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {
    /// Locate the project manifest file (`Cargo.toml`)
    #[command(name = "locate-project")]
    LocateProject(LocateProjectArgs),
    // ... other plumbing commands
}

#[derive(Debug, Args)]
pub(crate) struct LocateProjectArgs {
    /// Locate the workspace root `Cargo.toml` instead of the current package
    #[arg(long)]
    pub workspace: bool,

    /// The representation in which to print the project location
    #[arg(long = "message-format", value_enum, default_value_t = MessageFormat::Json)]
    message_format: MessageFormat,

    /// Path to the `Cargo.toml` file to start searching from
    #[arg(long = "manifest-path", value_name = "path")]
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum MessageFormat {
    Json,
    Plain,
}

impl Plumbing {
    pub(crate) fn exec(self) -> CargoResult<()> {
        match self {
            Plumbing::LocateProject(args) => exec_locate_project(args),
        }
    }
}

fn exec_locate_project(
    args: LocateProjectArgs
) -> CargoResult<()> {
    // Determine starting directory
    let start_dir = match args.manifest_path.clone() {
        Some(path) if path.is_file() && path.file_name() == Some("Cargo.toml".as_ref()) => {
            path.parent().unwrap().to_path_buf()
        }
        Some(path) => path,
        None => env::current_dir()?,
    };

    // Find the project manifest
    let manifest = find_manifest(&start_dir)?;

    // If workspace root requested, find workspace root
    let project_manifest = if args.workspace {
        let ws_dir = find_workspace_root(&manifest.parent().unwrap())?;
        ws_dir.join("Cargo.toml")
    } else {
        manifest
    };

    // Output
    match args.message_format {
        MessageFormat::Json => {
            let root_str = project_manifest.to_string_lossy();
            let obj = serde_json::json!({ "root": root_str });
            println!("{}", serde_json::to_string(&obj)?);
        }
        MessageFormat::Plain => {
            println!("{}", project_manifest.display());
        }
    }

    Ok(())
}

/// Search upward from `start` for a `Cargo.toml` file
fn find_manifest(start: &Path) -> CargoResult<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            return Ok(candidate);
        }
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            anyhow::bail!("failed to find Cargo.toml starting from {}", start.display());
        }
    }
}

/// Search upward from `start` for a `Cargo.toml` containing a `[workspace]` table
fn find_workspace_root(start: &Path) -> CargoResult<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            let content = fs::read_to_string(&candidate)?;
            if content.lines().any(|l| l.trim().starts_with("[workspace]")) {
                return Ok(dir);
            }
        }
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            anyhow::bail!(
                "failed to find workspace root Cargo.toml from {}",
                start.display()
            );
        }
    }
}
