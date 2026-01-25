//! Lint command implementation.

use adrs_core::{IssueSeverity, Repository, lint_adr, lint_all};
use anyhow::{Context, Result};
use std::path::Path;

/// Lint a single ADR or all ADRs.
pub fn lint(root: &Path, adr: Option<String>, all: bool) -> Result<()> {
    let repo =
        Repository::open(root).context("Failed to open repository. Have you run 'adrs init'?")?;

    let report = if all {
        lint_all(&repo).context("Failed to lint ADRs")?
    } else if let Some(adr_ref) = adr {
        // Parse as number or find by title
        let adr = if let Ok(num) = adr_ref.parse::<u32>() {
            repo.get(num)
                .context(format!("ADR {} not found", num))?
                .clone()
        } else {
            repo.find(&adr_ref)
                .context(format!("ADR '{}' not found", adr_ref))?
                .clone()
        };
        lint_adr(&adr).context("Failed to lint ADR")?
    } else {
        // No ADR specified and --all not set, show help
        anyhow::bail!("Specify an ADR number/title or use --all to lint all ADRs");
    };

    if report.issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }

    let error_count = report.count_by_severity(IssueSeverity::Error);
    let warning_count = report.count_by_severity(IssueSeverity::Warning);
    let info_count = report.count_by_severity(IssueSeverity::Info);

    for issue in &report.issues {
        let prefix = match issue.severity {
            IssueSeverity::Error => "error",
            IssueSeverity::Warning => "warning",
            IssueSeverity::Info => "info",
        };

        let location = match (&issue.path, issue.line, issue.adr_number) {
            (Some(path), Some(line), _) => {
                format!(" [{}:{}]", path.display(), line)
            }
            (Some(path), None, _) => format!(" [{}]", path.display()),
            (None, _, Some(num)) => format!(" [ADR {}]", num),
            _ => String::new(),
        };

        println!(
            "{}: [{}] {}{}",
            prefix, issue.rule_id, issue.message, location
        );
    }

    println!();
    println!(
        "Found {} error(s), {} warning(s), {} info(s)",
        error_count, warning_count, info_count
    );

    if report.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}
