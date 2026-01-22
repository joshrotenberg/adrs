//! List ADRs command.

use adrs_core::Repository;
use anyhow::{Context, Result};
use std::path::Path;

pub fn list(root: &Path) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let adrs = repo.list()?;
    for adr in adrs {
        if let Some(path) = &adr.path {
            println!("{}", path.display());
        } else {
            println!("{}", adr.filename());
        }
    }

    Ok(())
}
