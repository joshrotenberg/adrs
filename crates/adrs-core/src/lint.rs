//! ADR linting using mdbook-lint rules.
//!
//! This module provides unified linting for ADRs, combining per-file validation
//! (title format, required sections, date format) with repository-level checks
//! (sequential numbering, duplicate detection, broken links).

use crate::{Adr, Repository, Result};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{CollectionRule, Rule};
use mdbook_lint_rulesets::adr::{
    Adr001, Adr002, Adr003, Adr004, Adr005, Adr006, Adr007, Adr008, Adr009, Adr010, Adr011, Adr012,
    Adr013, Adr014, Adr015, Adr016, Adr017,
};
use std::path::PathBuf;

/// Severity level for lint issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational message.
    Info,
    /// Warning that should be addressed.
    Warning,
    /// Error that needs to be fixed.
    Error,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Info => write!(f, "info"),
            IssueSeverity::Warning => write!(f, "warning"),
            IssueSeverity::Error => write!(f, "error"),
        }
    }
}

impl From<mdbook_lint_core::Severity> for IssueSeverity {
    fn from(severity: mdbook_lint_core::Severity) -> Self {
        match severity {
            mdbook_lint_core::Severity::Error => IssueSeverity::Error,
            mdbook_lint_core::Severity::Warning => IssueSeverity::Warning,
            mdbook_lint_core::Severity::Info => IssueSeverity::Info,
        }
    }
}

/// A unified issue type for both per-file lint violations and repository-level diagnostics.
#[derive(Debug, Clone)]
pub struct Issue {
    /// The rule that produced this issue (e.g., "ADR001", "adr-title-format").
    pub rule_id: String,
    /// Human-readable rule name.
    pub rule_name: String,
    /// The severity of this issue.
    pub severity: IssueSeverity,
    /// A human-readable message describing the issue.
    pub message: String,
    /// The path to the affected file, if applicable.
    pub path: Option<PathBuf>,
    /// Line number (1-based), if applicable.
    pub line: Option<usize>,
    /// Column number (1-based), if applicable.
    pub column: Option<usize>,
    /// The ADR number, if applicable.
    pub adr_number: Option<u32>,
    /// Related ADR numbers (for issues involving multiple ADRs).
    pub related_adrs: Vec<u32>,
}

impl Issue {
    /// Create a new issue from an mdbook-lint violation.
    fn from_violation(
        violation: mdbook_lint_core::Violation,
        path: Option<PathBuf>,
        adr_number: Option<u32>,
    ) -> Self {
        Self {
            rule_id: violation.rule_id,
            rule_name: violation.rule_name,
            severity: violation.severity.into(),
            message: violation.message,
            path,
            line: Some(violation.line),
            column: Some(violation.column),
            adr_number,
            related_adrs: Vec::new(),
        }
    }
}

/// Results from linting.
#[derive(Debug, Default)]
pub struct LintReport {
    /// All issues found.
    pub issues: Vec<Issue>,
}

impl LintReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an issue to the report.
    pub fn add(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Warning)
    }

    /// Check if the report is clean (no warnings or errors).
    pub fn is_clean(&self) -> bool {
        !self.has_errors() && !self.has_warnings()
    }

    /// Get the count of issues by severity.
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    /// Sort issues by severity (errors first), then by path, then by line.
    pub fn sort(&mut self) {
        self.issues.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.path.cmp(&b.path))
                .then_with(|| a.line.cmp(&b.line))
        });
    }
}

/// Lint a single ADR file.
///
/// Runs all per-file lint rules against the ADR content.
pub fn lint_adr(adr: &Adr) -> Result<LintReport> {
    let mut report = LintReport::new();

    // Get the file content
    let Some(path) = &adr.path else {
        return Ok(report); // No path, nothing to lint
    };

    let content = std::fs::read_to_string(path)?;

    // Create mdbook-lint Document
    let doc = match Document::new(content, path.clone()) {
        Ok(d) => d,
        Err(e) => {
            report.add(Issue {
                rule_id: "parse-error".to_string(),
                rule_name: "parse-error".to_string(),
                severity: IssueSeverity::Error,
                message: format!("Failed to parse document: {e}"),
                path: Some(path.clone()),
                line: None,
                column: None,
                adr_number: Some(adr.number),
                related_adrs: Vec::new(),
            });
            return Ok(report);
        }
    };

    // Run all single-document rules
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(Adr001::default()),
        Box::new(Adr002::default()),
        Box::new(Adr003::default()),
        Box::new(Adr004::default()),
        Box::new(Adr005::default()),
        Box::new(Adr006::default()),
        Box::new(Adr007::default()),
        Box::new(Adr008::default()),
        Box::new(Adr009::default()),
        Box::new(Adr014::default()),
        Box::new(Adr015::default()),
        Box::new(Adr016::default()),
        Box::new(Adr017::default()),
    ];

    for rule in rules {
        match rule.check(&doc) {
            Ok(violations) => {
                for violation in violations {
                    report.add(Issue::from_violation(
                        violation,
                        Some(path.clone()),
                        Some(adr.number),
                    ));
                }
            }
            Err(e) => {
                report.add(Issue {
                    rule_id: rule.id().to_string(),
                    rule_name: rule.name().to_string(),
                    severity: IssueSeverity::Error,
                    message: format!("Rule failed: {e}"),
                    path: Some(path.clone()),
                    line: None,
                    column: None,
                    adr_number: Some(adr.number),
                    related_adrs: Vec::new(),
                });
            }
        }
    }

    Ok(report)
}

/// Lint all ADRs in a repository (per-file checks only).
pub fn lint_all(repo: &Repository) -> Result<LintReport> {
    let mut report = LintReport::new();
    let adrs = repo.list()?;

    for adr in &adrs {
        let adr_report = lint_adr(adr)?;
        report.issues.extend(adr_report.issues);
    }

    report.sort();
    Ok(report)
}

/// Run repository-level checks (collection rules).
///
/// These checks analyze the ADR set as a whole:
/// - Sequential numbering (ADR011)
/// - Duplicate numbers (ADR012)
/// - Broken links (ADR013)
/// - Superseded ADRs have replacements (ADR010)
pub fn check_repository(repo: &Repository) -> Result<LintReport> {
    let mut report = LintReport::new();
    let adrs = repo.list()?;

    // Build documents for collection rules
    let mut documents = Vec::new();
    for adr in &adrs {
        if let Some(path) = &adr.path {
            let content = std::fs::read_to_string(path)?;
            if let Ok(doc) = Document::new(content, path.clone()) {
                documents.push(doc);
            }
        }
    }

    // Run collection rules
    let collection_rules: Vec<Box<dyn CollectionRule>> = vec![
        Box::new(Adr010),
        Box::new(Adr011),
        Box::new(Adr012),
        Box::new(Adr013),
    ];

    for rule in collection_rules {
        match rule.check_collection(&documents) {
            Ok(violations) => {
                for violation in violations {
                    // Collection rule violations may have path in the message
                    // We need to parse it out or handle it differently
                    report.add(Issue {
                        rule_id: rule.id().to_string(),
                        rule_name: rule.name().to_string(),
                        severity: violation.severity.into(),
                        message: violation.message,
                        path: None, // Collection rules may span multiple files
                        line: if violation.line > 0 {
                            Some(violation.line)
                        } else {
                            None
                        },
                        column: if violation.column > 0 {
                            Some(violation.column)
                        } else {
                            None
                        },
                        adr_number: None,
                        related_adrs: Vec::new(),
                    });
                }
            }
            Err(e) => {
                report.add(Issue {
                    rule_id: rule.id().to_string(),
                    rule_name: rule.name().to_string(),
                    severity: IssueSeverity::Error,
                    message: format!("Rule failed: {e}"),
                    path: None,
                    line: None,
                    column: None,
                    adr_number: None,
                    related_adrs: Vec::new(),
                });
            }
        }
    }

    report.sort();
    Ok(report)
}

/// Run all checks: per-file lint + repository-level checks.
pub fn check_all(repo: &Repository) -> Result<LintReport> {
    let mut report = lint_all(repo)?;
    let repo_report = check_repository(repo)?;
    report.issues.extend(repo_report.issues);
    report.sort();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Adr;

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Error > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }

    #[test]
    fn test_lint_report_empty() {
        let report = LintReport::new();
        assert!(report.is_clean());
        assert!(!report.has_errors());
        assert!(!report.has_warnings());
    }

    #[test]
    fn test_lint_report_with_issues() {
        let mut report = LintReport::new();
        report.add(Issue {
            rule_id: "ADR001".to_string(),
            rule_name: "adr-title-format".to_string(),
            severity: IssueSeverity::Error,
            message: "Title format invalid".to_string(),
            path: Some(PathBuf::from("0001-test.md")),
            line: Some(1),
            column: Some(1),
            adr_number: Some(1),
            related_adrs: Vec::new(),
        });

        assert!(report.has_errors());
        assert!(!report.is_clean());
        assert_eq!(report.count_by_severity(IssueSeverity::Error), 1);
    }

    #[test]
    fn test_lint_valid_nygard_adr() {
        // Create a temporary file with valid Nygard format
        let content = r#"# 1. Record architecture decisions

Date: 2024-03-04

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records.

## Consequences

See Michael Nygard's article for details.
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir
            .path()
            .join("adr")
            .join("0001-record-architecture-decisions.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();

        let mut adr = Adr::new(1, "Record architecture decisions");
        adr.path = Some(path);

        let report = lint_adr(&adr).unwrap();

        // Print any issues for debugging
        for issue in &report.issues {
            println!(
                "{}: {} ({}:{})",
                issue.rule_id,
                issue.message,
                issue.line.unwrap_or(0),
                issue.column.unwrap_or(0)
            );
        }

        assert!(report.is_clean(), "Expected no issues for valid Nygard ADR");
    }

    #[test]
    fn test_lint_invalid_adr_missing_status() {
        let content = r#"# 1. Test decision

Date: 2024-03-04

## Context

Some context.

## Decision

Some decision.

## Consequences

Some consequences.
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("adr").join("0001-test-decision.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();

        let mut adr = Adr::new(1, "Test decision");
        adr.path = Some(path);

        let report = lint_adr(&adr).unwrap();

        // Should have at least one issue (missing status)
        assert!(
            !report.is_clean(),
            "Expected issues for ADR missing status section"
        );
        assert!(
            report.issues.iter().any(|i| i.rule_id == "ADR002"),
            "Expected ADR002 (missing status) violation"
        );
    }
}
