//! Initialize command.

use adrs_core::{ConfigWriteTarget, GitConfigScope, Repository};
use anyhow::{Context, Result};
use std::path::Path;
use std::path::PathBuf;

/// Initialize an ADR repository.
///
/// # Arguments
///
/// * `root` - The project root directory
/// * `directory` - The directory to store ADRs
/// * `ng` - Whether to use NextGen mode
/// * `git_config` - Whether to store config in .git/config instead of a file
pub fn init(root: &Path, directory: PathBuf, ng: bool, git_config: bool) -> Result<()> {
    // If git_config is requested, verify we're in a git repository
    if git_config && !root.join(".git").exists() {
        anyhow::bail!("Cannot use --git-config: not a git repository (no .git directory found)");
    }

    let repo = Repository::init(root, Some(directory.clone()), ng).with_context(|| {
        format!(
            "Failed to initialize ADR repository in {}",
            directory.display()
        )
    })?;

    // If --git-config was specified, save config to gitconfig and remove the file
    if git_config {
        // Save to gitconfig
        repo.config()
            .save_to(root, ConfigWriteTarget::GitConfig(GitConfigScope::Local))
            .with_context(|| "Failed to write configuration to .git/config")?;

        // Remove the file-based config that Repository::init created
        let toml_path = root.join("adrs.toml");
        let adr_dir_path = root.join(".adr-dir");

        if toml_path.exists() {
            std::fs::remove_file(&toml_path)
                .with_context(|| format!("Failed to remove {}", toml_path.display()))?;
        }
        if adr_dir_path.exists() {
            std::fs::remove_file(&adr_dir_path)
                .with_context(|| format!("Failed to remove {}", adr_dir_path.display()))?;
        }

        println!(
            "{} (config stored in .git/config)",
            repo.adr_path().display()
        );
    } else {
        // Normal output
        let adr_count = repo.list().map(|adrs| adrs.len()).unwrap_or(0);

        if adr_count > 1 {
            // More than just the initial ADR means we found existing ADRs
            println!(
                "{} ({} existing ADRs found)",
                repo.adr_path().display(),
                adr_count
            );
        } else {
            println!("{}", repo.adr_path().display());
        }
    }

    Ok(())
}
