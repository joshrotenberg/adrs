//! Config command.

use adrs_core::{ConfigSource, DiscoveredConfig};
use anyhow::Result;
use std::path::Path;

/// Show configuration with discovery information.
pub fn config_with_discovery(start_dir: &Path, discovered: Option<DiscoveredConfig>) -> Result<()> {
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
            println!("Mode: {:?}", disc.config.mode);
            if let Some(ref format) = disc.config.templates.format {
                println!("Template format: {}", format);
            }
            if let Some(ref custom) = disc.config.templates.custom {
                println!("Custom template: {}", custom.display());
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
