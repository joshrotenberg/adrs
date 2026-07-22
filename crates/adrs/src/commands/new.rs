//! New ADR command.

use adrs_core::{
    AdrStatus, Config, ConfigMode, LinkKind, Repository, Template, TemplateFormat, TemplateVariant,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub fn new(
    root: &Path,
    ng: bool,
    title: String,
    supersedes: Option<u32>,
    link: Option<String>,
    format: Option<String>,
    variant: Option<String>,
    template: Option<PathBuf>,
    status: Option<String>,
    tags: Option<Vec<String>>,
    deciders: Option<Vec<String>>,
    consulted: Option<Vec<String>>,
    informed: Option<Vec<String>>,
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
        var.parse::<TemplateVariant>().context(
            "Invalid template variant. Use 'full', 'minimal', 'bare', or 'bare-minimal'.",
        )?
    } else if let Some(ref var) = config.templates.variant {
        var.parse::<TemplateVariant>().context(
            "Invalid template variant in config. Use 'full', 'minimal', 'bare', or 'bare-minimal'.",
        )?
    } else {
        TemplateVariant::default()
    };

    // Parse CLI status override if specified.
    let cli_status = status
        .as_deref()
        .map(|s| s.parse::<AdrStatus>().unwrap_or(AdrStatus::Proposed));

    let mut repo = Repository::open(root)
        .context("ADR repository not found. Run 'adrs init' first.")?
        .with_template_format(template_format)
        .with_template_variant(template_variant);

    // Override mode if --ng flag is passed
    if ng {
        repo = repo.with_mode(ConfigMode::NextGen);
    }

    // Apply custom template: CLI --template > config custom > built-in format/variant
    if let Some(ref cli_path) = template {
        // CLI flag takes highest precedence
        let tmpl = Template::from_file(cli_path).context(format!(
            "Failed to load custom template from {}: check the path is correct",
            cli_path.display()
        ))?;
        repo = repo.with_custom_template(tmpl);
    } else if format.is_none()
        && variant.is_none()
        && let Some(ref custom_path) = config.templates.custom
    {
        // Config custom path applies when no CLI format/variant override is given
        let full_path = root.join(custom_path);
        let tmpl = Template::from_file(&full_path).context(format!(
            "Failed to load custom template from config: {}",
            custom_path.display()
        ))?;
        repo = repo.with_custom_template(tmpl);
    }

    let (mut adr, path) = if let Some(superseded) = supersedes {
        repo.supersede(&title, superseded)
            .context("Failed to create superseding ADR")?
    } else {
        repo.new_adr(&title).context("Failed to create new ADR")?
    };

    // Apply explicit CLI status if specified (superseding sets its own status).
    if supersedes.is_none()
        && let Some(status) = cli_status
    {
        adr.status = status;
        repo.update_metadata(&adr)
            .context("Failed to update ADR status")?;
    }

    // Effective ng mode: CLI flag --ng OR config mode = "ng"
    let is_ng = ng || config.is_next_gen();

    // Set tags if specified (requires ng mode for YAML frontmatter)
    if let Some(tag_list) = tags {
        if !is_ng {
            anyhow::bail!(
                "Tags require --ng mode (YAML frontmatter). Use: adrs --ng new --tags ... or set mode = \"ng\" in adrs.toml"
            );
        }
        if !tag_list.is_empty() {
            adr.set_tags(tag_list);
            repo.update_metadata(&adr)
                .context("Failed to update ADR tags")?;
        }
    }

    // Set MADR participant fields if specified (requires ng mode for YAML frontmatter).
    if deciders.is_some() || consulted.is_some() || informed.is_some() {
        if !is_ng {
            anyhow::bail!(
                "--deciders/--consulted/--informed require --ng mode (YAML frontmatter). Use: adrs --ng new --deciders ... or set mode = \"ng\" in adrs.toml"
            );
        }
        let mut changed = false;
        if let Some(list) = deciders {
            adr.set_decision_makers(list);
            changed = true;
        }
        if let Some(list) = consulted {
            adr.set_consulted(list);
            changed = true;
        }
        if let Some(list) = informed {
            adr.set_informed(list);
            changed = true;
        }
        if changed {
            repo.update_metadata(&adr)
                .context("Failed to update ADR participants")?;
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

    // Open in editor unless --no-edit flag or config no_edit is set.
    // Precedence: --no-edit CLI flag > no_edit config > built-in default (false).
    let skip_editor = no_edit || config.no_edit;
    if !skip_editor {
        edit::edit_file(&path).context("Failed to open editor")?;
    }

    println!("{}", path.display());
    Ok(())
}
