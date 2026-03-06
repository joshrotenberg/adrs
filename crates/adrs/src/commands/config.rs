//! Config command.

use crate::ConfigFormat;
use crate::output::{ConfigLayer, ConfigOutput, ConfigValues, OutputFormat, print_json};
use adrs_core::{Config, ConfigMode, ConfigSource, DiscoveredConfig};
use anyhow::{Context, Result};
use std::path::Path;

/// Show configuration with discovery information.
pub fn config_show(
    start_dir: &Path,
    discovered: Option<DiscoveredConfig>,
    verbose: bool,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => config_show_json(start_dir, discovered, verbose),
        OutputFormat::Plain => config_show_plain(start_dir, discovered, verbose),
    }
}

/// Show configuration as JSON.
fn config_show_json(
    start_dir: &Path,
    discovered: Option<DiscoveredConfig>,
    verbose: bool,
) -> Result<()> {
    match discovered {
        Some(disc) => {
            let source_str = match &disc.source {
                ConfigSource::Project(path) => path.display().to_string(),
                ConfigSource::Global(path) => format!("{}:global", path.display()),
                ConfigSource::Environment => "environment".to_string(),
                ConfigSource::Default => "defaults".to_string(),
            };

            let mode_str = match disc.config.mode {
                ConfigMode::Compatible => "compatible",
                ConfigMode::NextGen => "nextgen",
            };

            let layers = if verbose {
                collect_config_layers(&disc.root)
            } else {
                Vec::new()
            };

            let output = ConfigOutput {
                root: disc.root.display().to_string(),
                source: source_str,
                config: ConfigValues {
                    adr_dir: disc.config.adr_dir.display().to_string(),
                    mode: mode_str.to_string(),
                    template_format: disc.config.templates.format.clone(),
                    template_variant: disc.config.templates.variant.clone(),
                },
                layers,
            };

            print_json(&output)?;
        }
        None => {
            // No repository - output error state as JSON
            let output = serde_json::json!({
                "error": {
                    "code": "REPO_NOT_FOUND",
                    "message": "No ADR repository found",
                    "context": {
                        "search_dir": start_dir.display().to_string()
                    }
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

/// Collect config layer information for verbose JSON output.
fn collect_config_layers(root: &Path) -> Vec<ConfigLayer> {
    let mut layers = Vec::new();

    // Layer 1: Environment variables
    let has_env = std::env::var("ADR_DIRECTORY").is_ok();
    layers.push(ConfigLayer {
        source: "environment".to_string(),
        priority: 1,
        status: Some(if has_env { "active" } else { "not_set" }.to_string()),
    });

    // Layer 2: adrs.toml
    let toml_path = root.join("adrs.toml");
    layers.push(ConfigLayer {
        source: "adrs.toml".to_string(),
        priority: 2,
        status: Some(
            if toml_path.exists() {
                "active"
            } else {
                "not_found"
            }
            .to_string(),
        ),
    });

    // Layer 3: Legacy .adr-dir
    let adr_dir_path = root.join(".adr-dir");
    layers.push(ConfigLayer {
        source: ".adr-dir".to_string(),
        priority: 3,
        status: Some(
            if adr_dir_path.exists() {
                "active"
            } else {
                "not_found"
            }
            .to_string(),
        ),
    });

    // Layer 4: Global config
    let global_status = if let Some(path) = get_global_config_path() {
        if path.exists() { "active" } else { "not_found" }
    } else {
        "not_found"
    };
    layers.push(ConfigLayer {
        source: "global:config.toml".to_string(),
        priority: 4,
        status: Some(global_status.to_string()),
    });

    // Layer 5: Defaults
    layers.push(ConfigLayer {
        source: "defaults".to_string(),
        priority: 5,
        status: Some("active".to_string()),
    });

    layers
}

/// Show configuration as plain text.
fn config_show_plain(
    start_dir: &Path,
    discovered: Option<DiscoveredConfig>,
    verbose: bool,
) -> Result<()> {
    match discovered {
        Some(disc) => {
            println!("Project root: {}", disc.root.display());
            println!(
                "Config source: {}",
                match &disc.source {
                    ConfigSource::Project(path) => format!("{}", path.display()),
                    ConfigSource::Global(path) => format!("{} (global)", path.display()),
                    ConfigSource::Environment => "environment variable".to_string(),
                    ConfigSource::Default => "defaults".to_string(),
                }
            );
            println!("ADR directory: {}", disc.config.adr_dir.display());
            println!(
                "Full path: {}",
                disc.root.join(&disc.config.adr_dir).display()
            );
            println!(
                "Mode: {}",
                match disc.config.mode {
                    ConfigMode::Compatible => "Compatible",
                    ConfigMode::NextGen => "NextGen",
                }
            );
            if let Some(ref format) = disc.config.templates.format {
                println!("Template format: {}", format);
            }
            if let Some(ref variant) = disc.config.templates.variant {
                println!("Template variant: {}", variant);
            }
            if let Some(ref custom) = disc.config.templates.custom {
                println!("Custom template: {}", custom.display());
            }

            if verbose {
                println!();
                println!("Configuration layers (highest to lowest priority):");
                println!();
                print_verbose_layers(&disc.root);
            }
        }
        None => {
            println!("No ADR repository found.");
            println!("Search started from: {}", start_dir.display());
            println!();
            println!("Run 'adrs init' to create a new repository.");
        }
    }

    Ok(())
}

/// Print verbose information about each configuration layer.
fn print_verbose_layers(root: &Path) {
    // Layer 1: Environment variables
    println!("Layer 1: Environment variables");
    if let Ok(val) = std::env::var("ADR_DIRECTORY") {
        println!("  ADR_DIRECTORY = \"{}\"", val);
    } else {
        println!("  (not set)");
    }
    println!();

    // Layer 2: adrs.toml
    let toml_path = root.join("adrs.toml");
    println!("Layer 2: adrs.toml");
    if toml_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&toml_path) {
            for line in content.lines().take(10) {
                println!("  {}", line);
            }
            let line_count = content.lines().count();
            if line_count > 10 {
                println!("  ... ({} more lines)", line_count - 10);
            }
        }
    } else {
        println!("  (not found)");
    }
    println!();

    // Layer 3: .adr-dir
    let adr_dir_path = root.join(".adr-dir");
    println!("Layer 3: .adr-dir (legacy)");
    if adr_dir_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&adr_dir_path) {
            println!("  adr_dir = \"{}\"", content.trim());
        }
    } else {
        println!("  (not found)");
    }
    println!();

    // Layer 4: Global config
    let global_path = get_global_config_path();
    println!("Layer 4: Global config");
    if let Some(ref path) = global_path {
        if path.exists() {
            println!("  {}", path.display());
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines().take(5) {
                    println!("    {}", line);
                }
            }
        } else {
            println!("  {} (not found)", path.display());
        }
    } else {
        println!("  (path not available)");
    }
    println!();

    // Layer 5: Defaults
    println!("Layer 5: Defaults");
    println!("  adr_dir = \"doc/adr\"");
    println!("  mode = \"compatible\"");
}

/// Get the global config path.
fn get_global_config_path() -> Option<std::path::PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(
            std::path::PathBuf::from(xdg)
                .join("adrs")
                .join("config.toml"),
        );
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(
            std::path::PathBuf::from(home)
                .join(".config")
                .join("adrs")
                .join("config.toml"),
        );
    }
    None
}

/// Migrate configuration between formats.
pub fn config_migrate(
    discovered: &DiscoveredConfig,
    to: ConfigFormat,
    dry_run: bool,
) -> Result<()> {
    let root = &discovered.root;
    let config = &discovered.config;

    let toml_path = root.join("adrs.toml");
    let adr_dir_path = root.join(".adr-dir");

    match to {
        ConfigFormat::Toml => {
            // Migrate to adrs.toml
            if toml_path.exists() && !dry_run {
                anyhow::bail!(
                    "adrs.toml already exists at {}. Remove it first or use a different target.",
                    toml_path.display()
                );
            }

            // Build TOML content
            let new_config = Config {
                adr_dir: config.adr_dir.clone(),
                mode: config.mode,
                templates: config.templates.clone(),
            };
            let toml_content = toml::to_string_pretty(&new_config)
                .context("Failed to serialize configuration to TOML")?;

            if dry_run {
                println!("Would create: {}", toml_path.display());
                println!();
                println!("{}", toml_content);

                if adr_dir_path.exists() {
                    println!();
                    println!(
                        "Note: You may want to remove .adr-dir after migration: {}",
                        adr_dir_path.display()
                    );
                }
            } else {
                std::fs::write(&toml_path, &toml_content)
                    .with_context(|| format!("Failed to write {}", toml_path.display()))?;

                println!("Created: {}", toml_path.display());

                if adr_dir_path.exists() {
                    println!();
                    println!(
                        "Note: You may want to remove the old .adr-dir file: {}",
                        adr_dir_path.display()
                    );
                }
            }
        }
        ConfigFormat::AdrDir => {
            // Migrate to .adr-dir
            if adr_dir_path.exists() && !dry_run {
                anyhow::bail!(
                    ".adr-dir already exists at {}. Remove it first or use a different target.",
                    adr_dir_path.display()
                );
            }

            // Warn about lossy conversion
            let mut warnings = Vec::new();
            if config.mode != ConfigMode::Compatible {
                warnings.push("mode (will be lost - .adr-dir always implies Compatible mode)");
            }
            if config.templates.format.is_some() {
                warnings.push("templates.format");
            }
            if config.templates.variant.is_some() {
                warnings.push("templates.variant");
            }
            if config.templates.custom.is_some() {
                warnings.push("templates.custom");
            }

            if !warnings.is_empty() {
                println!("Warning: The following settings will be lost:");
                for w in &warnings {
                    println!("  - {}", w);
                }
                println!();
            }

            let content = config.adr_dir.display().to_string();

            if dry_run {
                println!("Would create: {}", adr_dir_path.display());
                println!();
                println!("{}", content);

                if toml_path.exists() {
                    println!();
                    println!(
                        "Note: You may want to remove adrs.toml after migration: {}",
                        toml_path.display()
                    );
                }
            } else {
                std::fs::write(&adr_dir_path, &content)
                    .with_context(|| format!("Failed to write {}", adr_dir_path.display()))?;

                println!("Created: {}", adr_dir_path.display());

                if toml_path.exists() {
                    println!();
                    println!(
                        "Note: You may want to remove the old adrs.toml file: {}",
                        toml_path.display()
                    );
                }
            }
        }
    }

    Ok(())
}
