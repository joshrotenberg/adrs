//! Config command.

use crate::ConfigFormat;
use adrs_core::{
    Config, ConfigMode, ConfigSource, DiscoveredConfig, GitConfigScope, global_config_dir,
};
use anyhow::{Context, Result};
use std::path::Path;

/// Show configuration with discovery information.
pub fn config_show(
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
                    ConfigSource::Environment => "ADRS_* environment variables".to_string(),
                    ConfigSource::EnvironmentConfig(path) => {
                        format!("ADRS_CONFIG={}", path.display())
                    }
                    ConfigSource::GitConfig(scope) => match scope {
                        GitConfigScope::Local => "gitconfig (local)".to_string(),
                        GitConfigScope::Global => "gitconfig (global)".to_string(),
                        GitConfigScope::System => "gitconfig (system)".to_string(),
                    },
                    ConfigSource::Project(path) => format!("{}", path.display()),
                    ConfigSource::LegacyProject(path) => {
                        format!("{} (legacy .adr-dir)", path.display())
                    }
                    ConfigSource::Global(path) => format!("{} (global)", path.display()),
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
///
/// Shows all 9 layers per ADR-0020 precedence order (highest to lowest).
fn print_verbose_layers(root: &Path) {
    // Layer 1: ADRS_* environment variables (highest priority)
    println!("Layer 1: ADRS_* environment variables (highest priority)");
    let mut has_env = false;
    if let Ok(val) = std::env::var("ADRS_DIR") {
        println!("  ADRS_DIR = \"{}\"", val);
        has_env = true;
    }
    if let Ok(val) = std::env::var("ADRS_MODE") {
        println!("  ADRS_MODE = \"{}\"", val);
        has_env = true;
    }
    if let Ok(val) = std::env::var("ADRS_TEMPLATE_FORMAT") {
        println!("  ADRS_TEMPLATE_FORMAT = \"{}\"", val);
        has_env = true;
    }
    if let Ok(val) = std::env::var("ADRS_TEMPLATE_VARIANT") {
        println!("  ADRS_TEMPLATE_VARIANT = \"{}\"", val);
        has_env = true;
    }
    // Check deprecated ADR_DIRECTORY
    if let Ok(val) = std::env::var("ADR_DIRECTORY") {
        println!("  ADR_DIRECTORY = \"{}\" (deprecated, use ADRS_DIR)", val);
        has_env = true;
    }
    if !has_env {
        println!("  (not set)");
    }
    println!();

    // Layer 2: ADRS_CONFIG environment variable
    println!("Layer 2: ADRS_CONFIG (explicit config file)");
    if let Ok(val) = std::env::var("ADRS_CONFIG") {
        println!("  ADRS_CONFIG = \"{}\"", val);
        if std::path::Path::new(&val).exists() {
            println!("  (file exists)");
        } else {
            println!("  (file not found!)");
        }
    } else {
        println!("  (not set)");
    }
    println!();

    // Layer 3: Local gitconfig (.git/config [adrs])
    println!("Layer 3: Local gitconfig (.git/config)");
    let local_git_config = root.join(".git/config");
    if local_git_config.exists() {
        println!("  {}", local_git_config.display());
        if has_adrs_section(&local_git_config) {
            print_gitconfig_section(root);
        } else {
            println!("    (no [adrs] section)");
        }
    } else {
        println!("  (not a git repository)");
    }
    println!();

    // Layer 4: Project adrs.toml
    let toml_path = root.join("adrs.toml");
    println!("Layer 4: Project adrs.toml");
    if toml_path.exists() {
        println!("  {}", toml_path.display());
        if let Ok(content) = std::fs::read_to_string(&toml_path) {
            for line in content.lines().take(10) {
                println!("    {}", line);
            }
            let line_count = content.lines().count();
            if line_count > 10 {
                println!("    ... ({} more lines)", line_count - 10);
            }
        }
    } else {
        println!("  (not found)");
    }
    println!();

    // Layer 5: Legacy .adr-dir
    let adr_dir_path = root.join(".adr-dir");
    println!("Layer 5: Legacy .adr-dir");
    if adr_dir_path.exists() {
        println!("  {}", adr_dir_path.display());
        if let Ok(content) = std::fs::read_to_string(&adr_dir_path) {
            println!("    directory = \"{}\"", content.trim());
        }
    } else {
        println!("  (not found)");
    }
    println!();

    // Layer 6: User global config (~/.config/adrs/config.toml)
    println!("Layer 6: User global config");
    if let Some(global_dir) = global_config_dir() {
        let global_path = global_dir.join("config.toml");
        if global_path.exists() {
            println!("  {}", global_path.display());
            if let Ok(content) = std::fs::read_to_string(&global_path) {
                for line in content.lines().take(5) {
                    println!("    {}", line);
                }
            }
        } else {
            println!("  {} (not found)", global_path.display());
        }
    } else {
        println!("  (path not available)");
    }
    println!();

    // Layer 7: Global gitconfig (~/.gitconfig [adrs])
    println!("Layer 7: Global gitconfig (~/.gitconfig)");
    if let Some(home) = dirs::home_dir() {
        let global_gitconfig = home.join(".gitconfig");
        if global_gitconfig.exists() {
            println!("  {}", global_gitconfig.display());
            if has_adrs_section(&global_gitconfig) {
                println!("    (has [adrs] section)");
            } else {
                println!("    (no [adrs] section)");
            }
        } else {
            println!("  {} (not found)", global_gitconfig.display());
        }
    } else {
        println!("  (home directory not available)");
    }
    println!();

    // Layer 8: System gitconfig (/etc/gitconfig [adrs])
    println!("Layer 8: System gitconfig (/etc/gitconfig)");
    #[cfg(unix)]
    let system_gitconfig = std::path::PathBuf::from("/etc/gitconfig");
    #[cfg(windows)]
    let system_gitconfig = std::path::PathBuf::from("C:\\ProgramData\\Git\\config");
    #[cfg(not(any(unix, windows)))]
    let system_gitconfig = std::path::PathBuf::from("/etc/gitconfig");

    if system_gitconfig.exists() {
        println!("  {}", system_gitconfig.display());
        if has_adrs_section(&system_gitconfig) {
            println!("    (has [adrs] section)");
        } else {
            println!("    (no [adrs] section)");
        }
    } else {
        println!("  {} (not found)", system_gitconfig.display());
    }
    println!();

    // Layer 9: Defaults (lowest priority)
    println!("Layer 9: Built-in defaults (lowest priority)");
    println!("  adr_dir = \"doc/adr\"");
    println!("  mode = \"compatible\"");
}

/// Check if a gitconfig file has an [adrs] section.
fn has_adrs_section(path: &std::path::Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(path) {
        content
            .lines()
            .any(|line| line.trim().to_lowercase().starts_with("[adrs]"))
    } else {
        false
    }
}

/// Print the [adrs] section from local gitconfig.
fn print_gitconfig_section(root: &Path) {
    let git_config_path = root.join(".git/config");
    if let Ok(content) = std::fs::read_to_string(&git_config_path) {
        let mut in_adrs_section = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.to_lowercase().starts_with("[adrs]") {
                in_adrs_section = true;
                continue;
            }
            if trimmed.starts_with('[') && in_adrs_section {
                break;
            }
            if in_adrs_section && !trimmed.is_empty() {
                println!("    {}", trimmed);
            }
        }
    }
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
