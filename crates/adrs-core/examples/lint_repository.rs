//! Run health checks over a repository and report any issues.
//!
//! `check_all` combines per-ADR linting with repository-wide checks (broken
//! links, duplicate numbers, numbering gaps, ...).
//!
//! Run with: `cargo run -p adrs-core --example lint_repository`

use adrs_core::{Repository, check_all};

fn main() -> adrs_core::Result<()> {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let repo = Repository::init(tmp.path(), None, false)?;
    repo.new_adr("Use gRPC for service-to-service calls")?;

    // `check_all` returns a report; a healthy repository yields an empty list.
    let report = check_all(&repo)?;
    if report.issues.is_empty() {
        println!("No issues found.");
    } else {
        println!("Found {} issue(s):", report.issues.len());
        for issue in &report.issues {
            println!(
                "  [{:?}] {}: {}",
                issue.severity, issue.rule_name, issue.message
            );
        }
    }

    Ok(())
}
