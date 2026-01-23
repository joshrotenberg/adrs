//! Import commands.

use adrs_core::{ImportOptions, import_to_directory};
use anyhow::{Context, Result};
use std::path::Path;

/// Import ADRs from JSON-ADR format.
///
/// If `dir` is provided, imports to that directory without requiring an adrs repository.
/// Otherwise, imports to the repository at `root`.
pub fn import_json(
    root: &Path,
    file: &Path,
    dir: Option<&Path>,
    overwrite: bool,
    renumber: bool,
    dry_run: bool,
    ng_mode: bool,
) -> Result<()> {
    // Read the JSON file
    let json_data = if file.to_str() == Some("-") {
        // Read from stdin
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    } else {
        std::fs::read_to_string(file).context(format!("Failed to read file: {}", file.display()))?
    };

    // Determine target directory
    let target_dir = if let Some(d) = dir {
        d.to_path_buf()
    } else {
        // Use repository's ADR directory
        let repo = adrs_core::Repository::open(root).context("Failed to open ADR repository")?;
        repo.config().adr_dir.clone()
    };

    let options = ImportOptions {
        overwrite,
        renumber,
        dry_run,
        ng_mode,
    };

    let result =
        import_to_directory(&json_data, &target_dir, &options).context("Failed to import ADRs")?;

    // Report results
    if dry_run {
        println!("Dry run - no files written\n");
        if result.imported > 0 {
            println!("Would import {} ADR(s):", result.imported);

            // Show renumbering mapping if applicable
            if renumber && !result.renumber_map.is_empty() {
                println!("\nRenumbering:");
                for (old_num, new_num) in &result.renumber_map {
                    println!("  ADR {} -> ADR {}", old_num, new_num);
                }
                println!();
            }

            for path in &result.files {
                println!("  {}", path.display());
            }
        }
    } else if result.imported > 0 {
        println!("Imported {} ADR(s):", result.imported);
        for path in &result.files {
            println!("  {}", path.display());
        }
    }

    if result.skipped > 0 {
        println!("\nSkipped {} ADR(s):", result.skipped);
        for warning in &result.warnings {
            println!("  {}", warning);
        }
    }

    if result.imported == 0 && result.skipped == 0 {
        println!("No ADRs to import.");
    }

    // Show renumbering info and warnings
    if renumber && result.imported > 0 && !dry_run {
        println!("\nNote: ADRs have been renumbered sequentially.");
        println!("Internal cross-references have been updated automatically.");

        // Count broken link warnings
        let broken_links: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("links to ADR") && w.contains("not in the import set"))
            .collect();

        if !broken_links.is_empty() {
            println!(
                "\nWarning: {} broken cross-reference(s) detected:",
                broken_links.len()
            );
            println!("These links reference ADRs that were not included in the import.");
            println!("You may need to manually fix these references.");
        }
    }

    Ok(())
}
