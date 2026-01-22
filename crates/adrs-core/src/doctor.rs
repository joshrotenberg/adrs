//! Health checks for ADR repositories.

use crate::{Adr, Repository, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// The severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational message.
    Info,
    /// Warning that should be addressed.
    Warning,
    /// Error that needs to be fixed.
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A diagnostic message from a health check.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The severity of this diagnostic.
    pub severity: Severity,
    /// The check that produced this diagnostic.
    pub check: Check,
    /// A human-readable message describing the issue.
    pub message: String,
    /// The path to the affected file, if applicable.
    pub path: Option<PathBuf>,
    /// The ADR number, if applicable.
    pub adr_number: Option<u32>,
}

impl Diagnostic {
    /// Create a new diagnostic.
    pub fn new(severity: Severity, check: Check, message: impl Into<String>) -> Self {
        Self {
            severity,
            check,
            message: message.into(),
            path: None,
            adr_number: None,
        }
    }

    /// Set the path for this diagnostic.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the ADR number for this diagnostic.
    pub fn with_adr(mut self, number: u32) -> Self {
        self.adr_number = Some(number);
        self
    }
}

/// The type of health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Check {
    /// Check for duplicate ADR numbers.
    DuplicateNumbers,
    /// Check for proper file naming (4-digit padded IDs).
    FileNaming,
    /// Check that all ADRs have a status.
    MissingStatus,
    /// Check that linked ADRs exist.
    BrokenLinks,
    /// Check for sequential numbering gaps.
    NumberingGaps,
    /// Check that superseded ADRs have a superseding link.
    SupersededLinks,
}

impl std::fmt::Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Check::DuplicateNumbers => write!(f, "duplicate-numbers"),
            Check::FileNaming => write!(f, "file-naming"),
            Check::MissingStatus => write!(f, "missing-status"),
            Check::BrokenLinks => write!(f, "broken-links"),
            Check::NumberingGaps => write!(f, "numbering-gaps"),
            Check::SupersededLinks => write!(f, "superseded-links"),
        }
    }
}

/// Results from running health checks.
#[derive(Debug, Default)]
pub struct DoctorReport {
    /// All diagnostics found.
    pub diagnostics: Vec<Diagnostic>,
}

impl DoctorReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic to the report.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning)
    }

    /// Check if the report is clean (no warnings or errors).
    pub fn is_healthy(&self) -> bool {
        !self.has_errors() && !self.has_warnings()
    }

    /// Get the count of diagnostics by severity.
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .count()
    }
}

/// Run all health checks on a repository.
pub fn check(repo: &Repository) -> Result<DoctorReport> {
    let adrs = repo.list()?;
    let mut report = DoctorReport::new();

    check_duplicate_numbers(&adrs, &mut report);
    check_file_naming(&adrs, &mut report);
    check_missing_status(&adrs, &mut report);
    check_broken_links(&adrs, &mut report);
    check_numbering_gaps(&adrs, &mut report);
    check_superseded_links(&adrs, &mut report);

    // Sort diagnostics by severity (errors first)
    report
        .diagnostics
        .sort_by(|a, b| b.severity.cmp(&a.severity));

    Ok(report)
}

/// Check for duplicate ADR numbers.
fn check_duplicate_numbers(adrs: &[Adr], report: &mut DoctorReport) {
    let mut seen: HashMap<u32, Vec<&Adr>> = HashMap::new();

    for adr in adrs {
        seen.entry(adr.number).or_default().push(adr);
    }

    for (number, duplicates) in seen {
        if duplicates.len() > 1 {
            let paths: Vec<_> = duplicates
                .iter()
                .filter_map(|a| a.path.as_ref().and_then(|p| p.file_name()))
                .map(|p| p.to_string_lossy())
                .collect();

            report.add(
                Diagnostic::new(
                    Severity::Error,
                    Check::DuplicateNumbers,
                    format!(
                        "ADR number {} is used by multiple files: {}",
                        number,
                        paths.join(", ")
                    ),
                )
                .with_adr(number),
            );
        }
    }
}

/// Check for proper file naming (4-digit padded IDs).
fn check_file_naming(adrs: &[Adr], report: &mut DoctorReport) {
    for adr in adrs {
        if let Some(path) = &adr.path
            && let Some(filename) = path.file_name().and_then(|f| f.to_str())
        {
            let expected_prefix = format!("{:04}-", adr.number);
            if !filename.starts_with(&expected_prefix) {
                report.add(
                    Diagnostic::new(
                        Severity::Warning,
                        Check::FileNaming,
                        format!(
                            "File '{}' should start with '{}'",
                            filename, expected_prefix
                        ),
                    )
                    .with_path(path)
                    .with_adr(adr.number),
                );
            }
        }
    }
}

/// Check that all ADRs have a status.
fn check_missing_status(adrs: &[Adr], report: &mut DoctorReport) {
    use crate::AdrStatus;

    for adr in adrs {
        // Check for custom empty status
        if let AdrStatus::Custom(s) = &adr.status
            && s.trim().is_empty()
        {
            report.add(
                Diagnostic::new(
                    Severity::Warning,
                    Check::MissingStatus,
                    format!("ADR {} '{}' has an empty status", adr.number, adr.title),
                )
                .with_path(adr.path.clone().unwrap_or_default())
                .with_adr(adr.number),
            );
        }
    }
}

/// Check that linked ADRs exist.
fn check_broken_links(adrs: &[Adr], report: &mut DoctorReport) {
    let existing_numbers: HashSet<u32> = adrs.iter().map(|a| a.number).collect();

    for adr in adrs {
        for link in &adr.links {
            if !existing_numbers.contains(&link.target) {
                report.add(
                    Diagnostic::new(
                        Severity::Error,
                        Check::BrokenLinks,
                        format!(
                            "ADR {} '{}' links to non-existent ADR {}",
                            adr.number, adr.title, link.target
                        ),
                    )
                    .with_path(adr.path.clone().unwrap_or_default())
                    .with_adr(adr.number),
                );
            }
        }
    }
}

/// Check for gaps in sequential numbering.
fn check_numbering_gaps(adrs: &[Adr], report: &mut DoctorReport) {
    if adrs.is_empty() {
        return;
    }

    let mut numbers: Vec<u32> = adrs.iter().map(|a| a.number).collect();
    numbers.sort();
    numbers.dedup();

    let min = *numbers.first().unwrap();
    let max = *numbers.last().unwrap();

    let missing: Vec<u32> = (min..=max).filter(|n| !numbers.contains(n)).collect();

    if !missing.is_empty() {
        let missing_str = if missing.len() <= 5 {
            missing
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            format!(
                "{}, ... ({} total)",
                missing[..3]
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                missing.len()
            )
        };

        report.add(Diagnostic::new(
            Severity::Info,
            Check::NumberingGaps,
            format!("Missing ADR numbers in sequence: {}", missing_str),
        ));
    }
}

/// Check that superseded ADRs have proper links.
fn check_superseded_links(adrs: &[Adr], report: &mut DoctorReport) {
    use crate::{AdrStatus, LinkKind};

    for adr in adrs {
        if adr.status == AdrStatus::Superseded {
            let has_superseded_by_link = adr
                .links
                .iter()
                .any(|link| link.kind == LinkKind::SupersededBy);

            if !has_superseded_by_link {
                report.add(
                    Diagnostic::new(
                        Severity::Warning,
                        Check::SupersededLinks,
                        format!(
                            "ADR {} '{}' has status 'Superseded' but no 'Superseded by' link",
                            adr.number, adr.title
                        ),
                    )
                    .with_path(adr.path.clone().unwrap_or_default())
                    .with_adr(adr.number),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AdrLink, AdrStatus, LinkKind};

    #[test]
    fn test_duplicate_numbers() {
        let adrs = vec![
            {
                let mut adr = Adr::new(1, "First");
                adr.path = Some(PathBuf::from("0001-first.md"));
                adr
            },
            {
                let mut adr = Adr::new(1, "Duplicate");
                adr.path = Some(PathBuf::from("0001-duplicate.md"));
                adr
            },
        ];

        let mut report = DoctorReport::new();
        check_duplicate_numbers(&adrs, &mut report);

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].severity, Severity::Error);
        assert_eq!(report.diagnostics[0].check, Check::DuplicateNumbers);
    }

    #[test]
    fn test_file_naming() {
        let adrs = vec![{
            let mut adr = Adr::new(1, "Test");
            adr.path = Some(PathBuf::from("1-test.md")); // Missing padding
            adr
        }];

        let mut report = DoctorReport::new();
        check_file_naming(&adrs, &mut report);

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].severity, Severity::Warning);
        assert_eq!(report.diagnostics[0].check, Check::FileNaming);
    }

    #[test]
    fn test_broken_links() {
        let adrs = vec![{
            let mut adr = Adr::new(1, "Test");
            adr.links.push(AdrLink {
                target: 99, // Doesn't exist
                kind: LinkKind::Supersedes,
                description: None,
            });
            adr
        }];

        let mut report = DoctorReport::new();
        check_broken_links(&adrs, &mut report);

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].severity, Severity::Error);
        assert_eq!(report.diagnostics[0].check, Check::BrokenLinks);
    }

    #[test]
    fn test_numbering_gaps() {
        let adrs = vec![
            Adr::new(1, "First"),
            Adr::new(3, "Third"), // Missing 2
            Adr::new(5, "Fifth"), // Missing 4
        ];

        let mut report = DoctorReport::new();
        check_numbering_gaps(&adrs, &mut report);

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].severity, Severity::Info);
        assert!(report.diagnostics[0].message.contains("2"));
        assert!(report.diagnostics[0].message.contains("4"));
    }

    #[test]
    fn test_superseded_without_link() {
        let adrs = vec![{
            let mut adr = Adr::new(1, "Old Decision");
            adr.status = AdrStatus::Superseded;
            // No SupersededBy link
            adr
        }];

        let mut report = DoctorReport::new();
        check_superseded_links(&adrs, &mut report);

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].severity, Severity::Warning);
        assert_eq!(report.diagnostics[0].check, Check::SupersededLinks);
    }

    #[test]
    fn test_healthy_repo() {
        let adrs = vec![
            {
                let mut adr = Adr::new(1, "First");
                adr.path = Some(PathBuf::from("0001-first.md"));
                adr.status = AdrStatus::Accepted;
                adr
            },
            {
                let mut adr = Adr::new(2, "Second");
                adr.path = Some(PathBuf::from("0002-second.md"));
                adr.status = AdrStatus::Proposed;
                adr
            },
        ];

        let mut report = DoctorReport::new();
        check_duplicate_numbers(&adrs, &mut report);
        check_file_naming(&adrs, &mut report);
        check_broken_links(&adrs, &mut report);
        check_numbering_gaps(&adrs, &mut report);
        check_superseded_links(&adrs, &mut report);

        assert!(report.is_healthy());
    }
}
