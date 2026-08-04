//! Renumber command for repairing duplicate or misassigned ADR numbers.

use adrs_core::Repository;
use anyhow::{Context, Result};
use std::path::Path;

/// Renumber an ADR, rewriting its filename, frontmatter `number` (nextgen
/// mode), H1 heading, and every inbound reference in the rest of the
/// repository.
pub fn renumber(root: &Path, from: u32, to: u32, file: Option<&Path>, dry_run: bool) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let result = repo
        .renumber(from, to, file, dry_run)
        .with_context(|| format!("Failed to renumber ADR {from} to {to}"))?;

    if result.no_op {
        println!("ADR {from} is already numbered {to}; nothing to do.");
        return Ok(());
    }

    let (verb_done, verb_pending) = if dry_run {
        ("Would rename", "Would update")
    } else {
        ("Renamed", "Updated")
    };

    if let Some((old_path, new_path)) = &result.renamed_file {
        println!(
            "{} {} -> {}",
            verb_done,
            old_path.display(),
            new_path.display()
        );
    }

    if result.frontmatter_updated {
        println!("  {verb_pending} frontmatter `number` to {to}");
    }
    if result.h1_updated {
        println!("  {verb_pending} H1 heading to number {to}");
    }

    if !result.updated_references.is_empty() {
        println!(
            "\n{} {} inbound reference(s):",
            verb_pending,
            result.updated_references.len()
        );
        for path in &result.updated_references {
            println!("  {}", path.display());
        }
    }

    if !result.ambiguous_references.is_empty() {
        println!(
            "\nNote: {} record(s) link to ADR {from} by number. Another record still\nhas that number, so the reference was left as-is rather than repointed:",
            result.ambiguous_references.len()
        );
        for path in &result.ambiguous_references {
            println!("  {}", path.display());
        }
    }

    if !result.prose_warnings.is_empty() {
        println!(
            "\nNote: {} file(s) outside the ADR directory mention the old filename (not rewritten):",
            result.prose_warnings.len()
        );
        for path in &result.prose_warnings {
            println!("  {}", path.display());
        }
    }

    if dry_run {
        println!("\nDry run - no files written");
    }

    Ok(())
}
