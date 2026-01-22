//! New ADR command.

use adrs_core::{AdrStatus, LinkKind, Repository, TemplateFormat, TemplateVariant};
use anyhow::{Context, Result};
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn new(
    root: &Path,
    _ng: bool,
    title: String,
    supersedes: Option<u32>,
    link: Option<String>,
    format: Option<String>,
    variant: Option<String>,
    status: Option<String>,
) -> Result<()> {
    // Parse template format if specified
    let template_format = if let Some(ref fmt) = format {
        fmt.parse::<TemplateFormat>()
            .context("Invalid template format. Use 'nygard' or 'madr'.")?
    } else {
        TemplateFormat::default()
    };

    // Parse template variant if specified
    let template_variant = if let Some(ref var) = variant {
        var.parse::<TemplateVariant>()
            .context("Invalid template variant. Use 'full', 'minimal', or 'bare'.")?
    } else {
        TemplateVariant::default()
    };

    // Parse status if specified
    let initial_status = if let Some(ref s) = status {
        s.parse::<AdrStatus>().unwrap_or(AdrStatus::Proposed)
    } else {
        AdrStatus::Proposed
    };

    let repo = Repository::open(root)
        .context("ADR repository not found. Run 'adrs init' first.")?
        .with_template_format(template_format)
        .with_template_variant(template_variant);

    let (mut adr, path) = if let Some(superseded) = supersedes {
        repo.supersede(&title, superseded)
            .context("Failed to create superseding ADR")?
    } else {
        repo.new_adr(&title).context("Failed to create new ADR")?
    };

    // Set custom status if specified (and not superseding, which sets its own status)
    if supersedes.is_none() && status.is_some() {
        adr.status = initial_status;
        repo.update(&adr).context("Failed to update ADR status")?;
    }

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
