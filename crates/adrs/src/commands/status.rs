//! Status command for changing ADR status.

use adrs_core::{AdrStatus, Repository};
use anyhow::{Context, Result};
use std::path::Path;

/// Change the status of an ADR.
pub fn status(
    root: &Path,
    adr_number: u32,
    new_status: &str,
    superseded_by: Option<u32>,
) -> Result<()> {
    let repo = Repository::open(root).context("Failed to open ADR repository")?;

    // Parse the status
    let status: AdrStatus = new_status.parse().unwrap_or_else(|_| {
        // If it doesn't match a known status, treat it as custom
        AdrStatus::Custom(new_status.to_string())
    });

    // Validate: --by only makes sense with superseded status
    if superseded_by.is_some() && !matches!(status, AdrStatus::Superseded) {
        anyhow::bail!("--by can only be used with 'superseded' status");
    }

    let path = repo
        .set_status(adr_number, status.clone(), superseded_by)
        .context(format!("Failed to update status for ADR {}", adr_number))?;

    // Report what we did
    match superseded_by {
        Some(by) => println!(
            "ADR {} status changed to {} (superseded by {})\n  {}",
            adr_number,
            status,
            by,
            path.display()
        ),
        None => println!(
            "ADR {} status changed to {}\n  {}",
            adr_number,
            status,
            path.display()
        ),
    }

    Ok(())
}
