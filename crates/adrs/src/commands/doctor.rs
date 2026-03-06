//! Doctor command implementation.

use crate::output::{DoctorIssue, DoctorOutput, DoctorSummary, OutputFormat, print_json};
use adrs_core::{IssueSeverity, Repository, check_all};
use anyhow::{Context, Result};
use std::path::Path;

/// Run health checks on the ADR repository.
///
/// Runs all lint checks (per-file and repository-level).
pub fn doctor(root: &Path, format: OutputFormat) -> Result<()> {
    let repo =
        Repository::open(root).context("Failed to open repository. Have you run 'adrs init'?")?;

    let report = check_all(&repo).context("Failed to run health checks")?;

    let error_count = report.count_by_severity(IssueSeverity::Error);
    let warning_count = report.count_by_severity(IssueSeverity::Warning);
    let info_count = report.count_by_severity(IssueSeverity::Info);

    match format {
        OutputFormat::Json => {
            let output = DoctorOutput {
                healthy: report.issues.is_empty(),
                issues: report
                    .issues
                    .iter()
                    .map(|issue| DoctorIssue {
                        severity: match issue.severity {
                            IssueSeverity::Error => "error".to_string(),
                            IssueSeverity::Warning => "warning".to_string(),
                            IssueSeverity::Info => "info".to_string(),
                        },
                        rule_id: issue.rule_id.clone(),
                        message: issue.message.clone(),
                        path: issue.path.as_ref().map(|p| p.display().to_string()),
                        line: issue.line.map(|l| l as u32),
                        adr_number: issue.adr_number,
                    })
                    .collect(),
                summary: DoctorSummary {
                    errors: error_count,
                    warnings: warning_count,
                    info: info_count,
                },
            };
            print_json(&output)?;

            if report.has_errors() {
                std::process::exit(1);
            }
        }
        OutputFormat::Plain => {
            if report.issues.is_empty() {
                println!("No issues found. Your ADR repository is healthy!");
                return Ok(());
            }

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
        }
    }

    Ok(())
}
