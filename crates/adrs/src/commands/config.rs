//! Config command.

use adrs_core::{Config, Repository};
use anyhow::Result;
use std::path::Path;

pub fn config(root: &Path) -> Result<()> {
    match Repository::open(root) {
        Ok(repo) => {
            let config = repo.config();
            println!("ADR directory: {}", repo.adr_path().display());
            println!("Mode: {:?}", config.mode);
            if let Some(ref format) = config.templates.format {
                println!("Template format: {}", format);
            }
            if let Some(ref custom) = config.templates.custom {
                println!("Custom template: {}", custom.display());
            }
        }
        Err(_) => {
            let config = Config::default();
            println!(
                "ADR directory: {} (not initialized)",
                config.adr_dir.display()
            );
            println!("Mode: {:?}", config.mode);
        }
    }

    Ok(())
}
