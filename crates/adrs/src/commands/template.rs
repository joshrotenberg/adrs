//! Template management commands.

use adrs_core::{Template, TemplateFormat, TemplateVariant};
use anyhow::Result;

/// List available templates.
pub fn list() -> Result<()> {
    println!("Built-in templates:");
    println!("  nygard     Classic adr-tools format (default)");
    println!("  madr       MADR 4.0.0 with structured metadata");
    println!();
    println!("Variants:");
    println!("  full           All sections with guidance text (default)");
    println!("  minimal        Core sections only, with guidance text");
    println!("  bare           All sections, but empty (no guidance)");
    println!("  bare-minimal   Core sections only, empty (no guidance)");
    println!();
    println!("Usage:");
    println!("  adrs new --format madr --variant minimal \"Title\"");
    println!("  adrs template show madr --variant bare");

    Ok(())
}

/// Show a template's content.
pub fn show(format: &str, variant: Option<&str>) -> Result<()> {
    let template_format: TemplateFormat = format
        .parse()
        .map_err(|_| anyhow::anyhow!("Unknown format '{}'. Use 'nygard' or 'madr'.", format))?;

    let template_variant: TemplateVariant = variant
        .map(|v| {
            v.parse().map_err(|_| {
                anyhow::anyhow!(
                    "Unknown variant '{}'. Use 'full', 'minimal', 'bare', or 'bare-minimal'.",
                    v
                )
            })
        })
        .transpose()?
        .unwrap_or_default();

    let template = Template::builtin_with_variant(template_format, template_variant);

    println!("# {} ({} variant)", template_format, template_variant);
    println!();
    println!("Template variables:");
    println!("  {{{{ number }}}}       ADR number");
    println!("  {{{{ title }}}}        ADR title");
    println!("  {{{{ date }}}}         Creation date (YYYY-MM-DD)");
    println!("  {{{{ status }}}}       Status (proposed, accepted, etc.)");
    println!("  {{{{ context }}}}      Context section content");
    println!("  {{{{ decision }}}}     Decision section content");
    println!("  {{{{ consequences }}}} Consequences section content");
    println!("  {{{{ links }}}}        Array of links to other ADRs");
    println!("  {{{{ is_ng }}}}        True if using next-gen mode");
    println!();
    if matches!(template_format, TemplateFormat::Madr) {
        println!("MADR-specific variables:");
        println!("  {{{{ decision_makers }}}} Array of decision makers");
        println!("  {{{{ consulted }}}}       Array of consulted parties");
        println!("  {{{{ informed }}}}        Array of informed parties");
        println!();
    }
    println!("---");
    println!();
    print!("{}", template.content());

    Ok(())
}
