//! Doctor command implementation.

use adrs_core::{IssueSeverity, Repository, check_all};
use anyhow::{Context, Result};
use std::path::Path;

/// Run health checks on the ADR repository.
///
/// Runs all lint checks (per-file and repository-level).
pub fn doctor(root: &Path) -> Result<()> {
    let repo =
        Repository::open(root).context("Failed to open repository. Have you run 'adrs init'?")?;

    let report = check_all(&repo).context("Failed to run health checks")?;

    if report.issues.is_empty() {
        println!("No issues found. Your ADR repository is healthy!");
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
