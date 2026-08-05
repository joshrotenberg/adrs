//! Initialize command.

use adrs_core::{Config, Repository, discover};
use anyhow::{Context, Result};
use std::path::Path;
use std::path::PathBuf;

/// Resolve the root to initialize in and the ADR directory to use.
///
/// An explicit `directory` argument always wins. Otherwise the ADR
/// directory is resolved through the same config discovery every other
/// command uses (`adrs.toml`, then `.adr-dir`, then `ADR_DIRECTORY`, then
/// the global config), falling back to `doc/adr` only when nothing is
/// configured.
fn resolve_init_target(root: &Path, directory: Option<PathBuf>) -> (PathBuf, PathBuf) {
    match directory {
        Some(dir) => (root.to_path_buf(), dir),
        None => match discover(root) {
            Ok(discovered) => {
                crate::warn_unknown_config_keys(&discovered);
                (discovered.root, discovered.config.adr_dir)
            }
            Err(_) => (root.to_path_buf(), Config::default().adr_dir),
        },
    }
}

pub fn init(root: &Path, directory: Option<PathBuf>, ng: bool) -> Result<()> {
    let (init_root, adr_dir) = resolve_init_target(root, directory);

    let repo = Repository::init(&init_root, Some(adr_dir.clone()), ng).with_context(|| {
        format!(
            "Failed to initialize ADR repository in {}",
            adr_dir.display()
        )
    })?;

    // Check how many ADRs exist
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

    Ok(())
}
