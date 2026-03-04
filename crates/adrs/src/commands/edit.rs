//! Edit ADR command.

use adrs_core::Repository;
use anyhow::{Context, Result};
use std::path::Path;

pub fn edit(root: &Path, query: &str) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let adr = repo.find(query).context("ADR not found")?;
    let path = adr
        .path
        .clone()
        .unwrap_or_else(|| repo.adr_path().join(adr.filename()));

    edit::edit_file(&path).context("Failed to open editor")?;

    println!("{}", path.display());
    Ok(())
}
