//! New ADR command.

use adrs_core::{
    AdrStatus, Config, LinkKind, Repository, Template, TemplateFormat, TemplateVariant,
};
use anyhow::{Context, Result};
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn new(
    root: &Path,
    ng: bool,
    title: String,
    supersedes: Option<u32>,
    link: Option<String>,
    format: Option<String>,
    variant: Option<String>,
    status: Option<String>,
    tags: Option<Vec<String>>,
    no_edit: bool,
    config: &Config,
) -> Result<()> {
    // Parse template format: CLI arg > config > default
    let template_format = if let Some(ref fmt) = format {
        fmt.parse::<TemplateFormat>()
            .context("Invalid template format. Use 'nygard' or 'madr'.")?
    } else if let Some(ref fmt) = config.templates.format {
        fmt.parse::<TemplateFormat>()
            .context("Invalid template format in config. Use 'nygard' or 'madr'.")?
    } else {
        TemplateFormat::default()
    };

    // Parse template variant: CLI arg > config > default
    let template_variant = if let Some(ref var) = variant {
        var.parse::<TemplateVariant>()
            .context("Invalid template variant. Use 'full', 'minimal', or 'bare'.")?
    } else if let Some(ref var) = config.templates.variant {
        var.parse::<TemplateVariant>()
            .context("Invalid template variant in config. Use 'full', 'minimal', or 'bare'.")?
    } else {
        TemplateVariant::default()
    };

    // Parse status if specified
    let initial_status = if let Some(ref s) = status {
        s.parse::<AdrStatus>().unwrap_or(AdrStatus::Proposed)
    } else {
        AdrStatus::Proposed
    };

    let mut repo = Repository::open(root)
        .context("ADR repository not found. Run 'adrs init' first.")?
        .with_template_format(template_format)
        .with_template_variant(template_variant);

    // Load custom template from config if set (and no format/variant CLI override)
    if format.is_none()
        && variant.is_none()
        && let Some(ref custom_path) = config.templates.custom
    {
        let full_path = root.join(custom_path);
        let template = Template::from_file(&full_path).context(format!(
            "Failed to load custom template from config: {}",
            custom_path.display()
        ))?;
        repo = repo.with_custom_template(template);
    }

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

    // Set tags if specified (requires --ng mode for YAML frontmatter)
    if let Some(tag_list) = tags {
        if !ng {
            anyhow::bail!(
                "Tags require --ng mode (YAML frontmatter). Use: adrs --ng new --tags ..."
            );
        }
        if !tag_list.is_empty() {
            adr.set_tags(tag_list);
            repo.update(&adr).context("Failed to update ADR tags")?;
        }
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

    // Open in editor unless --no-edit was specified
    if !no_edit {
        let content = repo.read_content(&adr)?;
        let edited = edit::edit(&content).context("Failed to open editor")?;
        repo.write_content(&adr, &edited)?;
    }

    println!("{}", path.display());
    Ok(())
}
