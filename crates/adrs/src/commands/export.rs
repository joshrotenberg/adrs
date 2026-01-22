//! Export commands.

use adrs_core::{Repository, export_adr, export_directory, export_repository};
use anyhow::{Context, Result};
use std::path::Path;

/// Export ADRs to JSON-ADR format.
///
/// If `dir` is provided, exports from that directory without requiring an adrs repository.
/// Otherwise, exports from the repository at `root`.
pub fn export_json(
    root: &Path,
    adr_number: Option<u32>,
    dir: Option<&Path>,
    pretty: bool,
) -> Result<()> {
    let json = if let Some(dir_path) = dir {
        // Export from arbitrary directory (no repo required)
        if adr_number.is_some() {
            anyhow::bail!("Cannot specify both --dir and an ADR number");
        }

        let export = export_directory(dir_path).context(format!(
            "Failed to export from directory: {}",
            dir_path.display()
        ))?;

        if pretty {
            serde_json::to_string_pretty(&export)?
        } else {
            serde_json::to_string(&export)?
        }
    } else {
        // Export from repository
        let repo = Repository::open(root).context("Failed to open ADR repository")?;

        if let Some(number) = adr_number {
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
        }
    };

    println!("{}", json);
    Ok(())
}
