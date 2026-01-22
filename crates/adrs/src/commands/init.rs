//! Initialize command.

use adrs_core::Repository;
use anyhow::{Context, Result};
use std::path::Path;
use std::path::PathBuf;

pub fn init(root: &Path, directory: PathBuf, ng: bool) -> Result<()> {
    let repo = Repository::init(root, Some(directory.clone()), ng).with_context(|| {
        format!(
            "Failed to initialize ADR repository in {}",
            directory.display()
        )
    })?;

    println!("{}", repo.adr_path().display());
    Ok(())
}
