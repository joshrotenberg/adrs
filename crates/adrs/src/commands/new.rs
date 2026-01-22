//! New ADR command.

use adrs_core::{LinkKind, Repository};
use anyhow::{Context, Result};
use std::path::Path;

pub fn new(
    root: &Path,
    _ng: bool,
    title: String,
    supersedes: Option<u32>,
    link: Option<String>,
) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let (adr, path) = if let Some(superseded) = supersedes {
        repo.supersede(&title, superseded)
            .context("Failed to create superseding ADR")?
    } else {
        repo.new_adr(&title).context("Failed to create new ADR")?
    };

    // Handle linking if specified
    if let Some(link_spec) = link {
        let parts: Vec<&str> = link_spec.split(':').collect();
        if parts.len() == 3 {
            let target: u32 = parts[0]
                .parse()
                .context("Invalid target ADR number in link")?;
            let kind: LinkKind = parts[1].parse().unwrap_or(LinkKind::RelatesTo);
            let reverse_kind: LinkKind = parts[2].parse().unwrap_or(LinkKind::RelatesTo);

            repo.link(adr.number, target, kind, reverse_kind)
                .context("Failed to create link")?;
        }
    }

    // Open in editor
    let content = repo.read_content(&adr)?;
    let edited = edit::edit(&content).context("Failed to open editor")?;
    repo.write_content(&adr, &edited)?;

    println!("{}", path.display());
    Ok(())
}
