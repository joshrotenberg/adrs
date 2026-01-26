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
