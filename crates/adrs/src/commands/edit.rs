//! Edit ADR command.

use adrs_core::Repository;
use anyhow::{Context, Result};
use std::path::Path;

pub fn edit(root: &Path, query: &str) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let adr = repo.find(query).context("ADR not found")?;
    let content = repo.read_content(&adr)?;

    let edited = edit::edit(&content).context("Failed to open editor")?;
    let path = repo.write_content(&adr, &edited)?;

    println!("{}", path.display());
    Ok(())
}
