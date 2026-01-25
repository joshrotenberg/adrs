//! Export commands.

use adrs_core::{JsonAdr, Repository, export_adr, export_directory, export_repository};
use anyhow::{Context, Result};
use std::path::Path;

/// Export ADRs to JSON-ADR format.
///
/// If `dir` is provided, exports from that directory without requiring an adrs repository.
/// Otherwise, exports from the repository at `root`.
///
/// Options:
/// - `metadata_only`: If true, excludes content fields (context, decision, consequences)
///   and sets source_uri based on base_url.
/// - `base_url`: Base URL for constructing source_uri values.
pub fn export_json(
    root: &Path,
    adr_number: Option<u32>,
    dir: Option<&Path>,
    pretty: bool,
    metadata_only: bool,
    base_url: Option<String>,
) -> Result<()> {
    let json = if let Some(dir_path) = dir {
        // Export from arbitrary directory (no repo required)
        if adr_number.is_some() {
            anyhow::bail!("Cannot specify both --dir and an ADR number");
        }

        let mut export = export_directory(dir_path).context(format!(
            "Failed to export from directory: {}",
            dir_path.display()
        ))?;

        if metadata_only || base_url.is_some() {
            for adr in &mut export.adrs {
                apply_metadata_options(adr, metadata_only, &base_url);
            }
        }

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
            let mut json_adr = export_adr(&adr);

            if metadata_only || base_url.is_some() {
                apply_metadata_options(&mut json_adr, metadata_only, &base_url);
            }

            if pretty {
                serde_json::to_string_pretty(&json_adr)?
            } else {
                serde_json::to_string(&json_adr)?
            }
        } else {
            // Export all ADRs
            let mut export = export_repository(&repo)?;

            if metadata_only || base_url.is_some() {
                for adr in &mut export.adrs {
                    apply_metadata_options(adr, metadata_only, &base_url);
                }
            }

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

/// Apply metadata-only options to a JSON ADR.
fn apply_metadata_options(adr: &mut JsonAdr, metadata_only: bool, base_url: &Option<String>) {
    // Set source_uri from base_url and path
    if let Some(base) = base_url
        && let Some(path) = &adr.path
    {
        // Extract just the filename from the path
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);

        // Construct the full URI
        let base_trimmed = base.trim_end_matches('/');
        adr.source_uri = Some(format!("{}/{}", base_trimmed, filename));
    }

    // Clear content fields if metadata_only
    if metadata_only {
        adr.context = None;
        adr.decision = None;
        adr.consequences = None;
        adr.confirmation = None;
        adr.decision_drivers.clear();
        adr.considered_options.clear();
        adr.custom_sections.clear();
    }
}
