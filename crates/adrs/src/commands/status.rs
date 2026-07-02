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
    // Reject empty or whitespace-only status values. Writing one produces a
    // YAML null in the frontmatter, which fails to deserialize and silently
    // drops the ADR from `repo.list()` (see issue #305).
    if new_status.trim().is_empty() {
        anyhow::bail!("Status cannot be empty or whitespace-only");
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_status_rejects_empty_and_whitespace() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        repo.new_adr("Test decision").unwrap();

        for bad in ["", " ", "   ", "\t"] {
            let err = status(temp.path(), 1, bad, None).unwrap_err();
            assert!(
                err.to_string().contains("empty or whitespace-only"),
                "expected empty-status error for {:?}, got: {}",
                bad,
                err
            );
        }

        // The ADR must remain findable and its status unchanged.
        let adr = repo.get(1).unwrap();
        assert_eq!(adr.status, AdrStatus::Proposed);
    }
}
