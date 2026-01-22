//! Export commands.

use adrs_core::{Repository, export_adr, export_repository};
use anyhow::{Context, Result};
use std::path::Path;

/// Export ADRs to JSON-ADR format.
pub fn export_json(root: &Path, adr_number: Option<u32>, pretty: bool) -> Result<()> {
    let repo = Repository::open(root).context("Failed to open ADR repository")?;

    let json = if let Some(number) = adr_number {
        // Export single ADR
        let adr = repo
            .get(number)
            .context(format!("ADR {} not found", number))?;
        let json_adr = export_adr(&adr);

        if pretty {
            serde_json::to_string_pretty(&json_adr)?
        } else {
            serde_json::to_string(&json_adr)?
        }
    } else {
        // Export all ADRs
        let export = export_repository(&repo)?;

        if pretty {
            serde_json::to_string_pretty(&export)?
        } else {
            serde_json::to_string(&export)?
        }
    };

    println!("{}", json);
    Ok(())
}
