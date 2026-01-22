//! Doctor command implementation.

use adrs_core::{Repository, Severity, doctor_check};
use anyhow::{Context, Result};
use std::path::Path;

/// Run health checks on the ADR repository.
pub fn doctor(root: &Path) -> Result<()> {
    let repo =
        Repository::open(root).context("Failed to open repository. Have you run 'adrs init'?")?;

    let report = doctor_check(&repo).context("Failed to run health checks")?;

    if report.diagnostics.is_empty() {
        println!("No issues found. Your ADR repository is healthy!");
        return Ok(());
    }

    let error_count = report.count_by_severity(Severity::Error);
    let warning_count = report.count_by_severity(Severity::Warning);
    let info_count = report.count_by_severity(Severity::Info);

    for diagnostic in &report.diagnostics {
        let prefix = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        let location = match (&diagnostic.path, diagnostic.adr_number) {
            (Some(path), _) => format!(" [{}]", path.display()),
            (None, Some(num)) => format!(" [ADR {}]", num),
            _ => String::new(),
        };

        println!("{}: {}{}", prefix, diagnostic.message, location);
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
