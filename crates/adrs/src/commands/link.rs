//! Link ADRs command.

use adrs_core::{LinkKind, Repository};
use anyhow::{Context, Result};
use std::path::Path;

pub fn link(
    root: &Path,
    source: u32,
    link_kind: &str,
    target: u32,
    reverse_kind: &str,
) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let source_kind: LinkKind = link_kind.parse().unwrap_or(LinkKind::RelatesTo);
    let target_kind: LinkKind = reverse_kind.parse().unwrap_or(LinkKind::RelatesTo);

    repo.link(source, target, source_kind, target_kind)?;

    let source_adr = repo.get(source)?;
    let target_adr = repo.get(target)?;

    if let Some(path) = &source_adr.path {
        println!("{}", path.display());
    }
    if let Some(path) = &target_adr.path {
        println!("{}", path.display());
    }

    Ok(())
}
