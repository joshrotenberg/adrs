//! ADR linting using mdbook-lint rules.
//!
//! This module provides unified linting for ADRs, combining per-file validation
//! (title format, required sections, date format) with repository-level checks
//! (sequential numbering, duplicate detection, broken links).

use crate::{Adr, Repository, Result};
use globset::Glob;
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{CollectionRule, Rule};
use mdbook_lint_rulesets::adr::{
    Adr001, Adr002, Adr003, Adr004, Adr005, Adr006, Adr007, Adr008, Adr009, Adr010, Adr011, Adr012,
    Adr013, Adr014, Adr015, Adr016, Adr017, AdrFormat,
};
use std::collections::HashSet;
use std::path::PathBuf;

/// Rule IDs and rule names that are *always* produced without a path.
///
/// These are the upstream collection rules whose only source is
/// `check_repository`'s `CollectionRule::check_collection` loop (`path: None`
/// there -- see the comment on that loop). `ADR013` / `adr-valid-adr-links` is
/// deliberately excluded: that rule id is also used by the frontmatter
/// broken-link check above the collection-rule loop, which *does* set a path,
/// so a `[[doctor.ignore_path]]` entry naming it can fire for that source even
/// though it can never match the collection-rule source.
///
/// Used to warn when a `[[doctor.ignore_path]]` entry names a rule that can
/// never be suppressed by path (issue #365).
const ALWAYS_PATHLESS_RULES: &[&str] = &[
    "ADR010",
    "adr-superseded-has-replacement",
    "ADR011",
    "adr-sequential-numbering",
    "ADR012",
    "adr-no-duplicate-numbers",
];

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

/// Detect an ADR document's format from its section headings.
///
/// The mdbook-lint ADR rules auto-detect format from YAML frontmatter alone,
/// treating every frontmatter document as MADR. That misclassifies a
/// frontmatter-backed Nygard ADR (as produced by `adrs --ng init`), which then
/// fails the MADR section rules (#348). This detects the format the way a reader
/// would: from the headings actually present.
///
/// - A MADR-specific H2 (`## Context and Problem Statement`, `## Decision
///   Outcome`, or `## Considered Options`) means [`AdrFormat::Madr4`].
/// - Otherwise a Nygard H2 (`## Context` or `## Decision`) means
///   [`AdrFormat::Nygard`].
/// - With neither present, fall back to [`AdrFormat::Auto`] so the rules apply
///   their own frontmatter-based heuristic unchanged.
fn detect_adr_format(content: &str) -> AdrFormat {
    let mut has_nygard = false;

    for line in content.lines() {
        let Some(heading) = line.strip_prefix("## ") else {
            continue;
        };
        let heading = heading.trim();

        if heading.eq_ignore_ascii_case("Context and Problem Statement")
            || heading.eq_ignore_ascii_case("Decision Outcome")
            || heading.eq_ignore_ascii_case("Considered Options")
        {
            return AdrFormat::Madr4;
        }

        if heading.eq_ignore_ascii_case("Context") || heading.eq_ignore_ascii_case("Decision") {
            has_nygard = true;
        }
    }

    if has_nygard {
        AdrFormat::Nygard
    } else {
        AdrFormat::Auto
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

    // Detect the document's actual format from its section headings, not from
    // the mere presence of YAML frontmatter. The mdbook-lint ADR rules default
    // to `AdrFormat::Auto`, which classifies any frontmatter document as MADR
    // (see #348: `adrs --ng init` writes frontmatter + Nygard headings, which
    // Auto then flags as missing the MADR `## Context and Problem Statement` /
    // `## Decision Outcome` sections). Pinning the format-sensitive rules to the
    // format we detect from the headings keeps a frontmatter-backed Nygard ADR
    // valid while still validating genuine MADR documents.
    let format = detect_adr_format(&doc.content);

    // Run all single-document rules
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(Adr001::default()),
        Box::new(Adr002::default()),
        Box::new(Adr003::default()),
        Box::new(Adr004::with_format(format)),
        Box::new(Adr005::with_format(format)),
        Box::new(Adr006::with_format(format)),
        Box::new(Adr007::default()),
        Box::new(Adr008::default()),
        Box::new(Adr009::default()),
        Box::new(Adr014::default()),
        Box::new(Adr015::default()),
        Box::new(Adr016::default()),
        Box::new(Adr017::with_format(format)),
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
/// - Asymmetric links (`asymmetric-link`)
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

    // Resolve frontmatter `links[].target` against the ADR numbers actually
    // present in the repository. This is distinct from the upstream ADR013
    // rule below, which only checks markdown link *filenames* in the
    // rendered body and has no notion of frontmatter (#355). A frontmatter
    // link is a structured reference to an ADR number, so an unresolvable
    // target is unambiguously broken and reported at `Error` rather than the
    // upstream rule's `Warning`.
    //
    // Gated to nextgen mode: in compatible mode, `adr.links` is populated by
    // parsing legacy body syntax like `Supersedes [1. Title](0001-title.md)`
    // (see parse.rs), not frontmatter, and that path is already covered by
    // the upstream markdown-filename check below. Checking it again here
    // would change compatible-mode severity/exit-code behavior, which is out
    // of scope for this fix.
    //
    // Track the prefix of the message the upstream ADR013 rule would emit for
    // each broken link so its warning can be suppressed once our error covers
    // the same broken link. Nygard-family templates render a link both in
    // frontmatter and as a body markdown link, so without this a single
    // broken link would otherwise produce two ADR013 issues for the same
    // record.
    //
    // The prefix stops at the target's zero-padded number rather than
    // spelling out a whole filename, because the body filename varies: an
    // unresolvable target renders as the `{:04}-....md` fallback (see
    // template.rs's `resolve_link_titles`), but a link rendered while its
    // target still existed keeps that target's real filename, which is the
    // case #355 was found through. Anchoring on the record's path and the
    // padded number matches both without matching a different record or a
    // different target.
    let mut broken_link_fragments: Vec<String> = Vec::new();

    if repo.config().is_next_gen() {
        let existing_numbers: std::collections::HashSet<u32> =
            adrs.iter().map(|a| a.number).collect();

        for adr in &adrs {
            for link in &adr.links {
                if existing_numbers.contains(&link.target) {
                    continue;
                }

                let path = adr.path.clone().unwrap_or_default();
                broken_link_fragments.push(format!(
                    "{}: Link to '{:04}",
                    path.display(),
                    link.target
                ));

                report.add(Issue {
                    rule_id: "ADR013".to_string(),
                    rule_name: "adr-valid-adr-links".to_string(),
                    severity: IssueSeverity::Error,
                    message: format!(
                        "ADR {} '{}' links to non-existent ADR {}",
                        adr.number, adr.title, link.target
                    ),
                    path: adr.path.clone(),
                    line: None,
                    column: None,
                    adr_number: Some(adr.number),
                    related_adrs: Vec::new(),
                });
            }
        }
    }

    // Check that every link is reciprocated (#357). `adrs link` maintains
    // both halves of a relationship by construction, so an asymmetric pair
    // is evidence that something outside the tool edited the record -- a
    // hand edit, a renumber, a badly resolved merge. For each link A -> B,
    // require that B carries some link back to A.
    //
    // The back-link is matched by target only, not by `LinkKind::reverse()`'s
    // exact kind. `adrs link` accepts an explicit `reverse_kind` override
    // (see commands/link.rs), so requiring the derived kind exactly would
    // flag the tool's own supported output as corruption; asking only "does
    // B link back to A" agrees with the tool by construction.
    //
    // Links whose target does not exist are skipped: that is a broken link,
    // already reported as ADR013 above (frontmatter) or below (markdown
    // body). Reporting it again as asymmetry would give two diagnostics for
    // one problem.
    //
    // Unlike the frontmatter check above, this is not gated to nextgen mode:
    // `adr.links` is populated in compatible mode by parsing legacy body
    // syntax (see parse.rs) and in nextgen mode from frontmatter, and there
    // is no upstream rule here to conflict with.
    let adrs_by_number: std::collections::HashMap<u32, &Adr> =
        adrs.iter().map(|a| (a.number, a)).collect();

    for adr in &adrs {
        for link in &adr.links {
            let Some(target_adr) = adrs_by_number.get(&link.target) else {
                continue;
            };

            let has_back_link = target_adr
                .links
                .iter()
                .any(|back| back.target == adr.number);

            if !has_back_link {
                report.add(Issue {
                    rule_id: "asymmetric-link".to_string(),
                    rule_name: "adr-asymmetric-link".to_string(),
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "ADR {} '{}' links to ADR {} as '{}' but ADR {} has no link back to ADR {}",
                        adr.number, adr.title, link.target, link.kind, link.target, adr.number
                    ),
                    path: adr.path.clone(),
                    line: None,
                    column: None,
                    adr_number: Some(adr.number),
                    related_adrs: Vec::new(),
                });
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
                    // Suppress the upstream markdown-filename ADR013 warning
                    // when the frontmatter check above already reported the
                    // same broken link as an error (see comment above).
                    if violation.rule_id == "ADR013"
                        && broken_link_fragments
                            .iter()
                            .any(|fragment| violation.message.contains(fragment.as_str()))
                    {
                        continue;
                    }

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

/// A compiled `[[doctor.ignore_path]]` entry, ready to match against issue paths.
struct CompiledIgnorePath {
    matcher: globset::GlobMatcher,
    rules: HashSet<String>,
}

/// Run all checks and filter out issues matching ignored rule IDs/names,
/// repository-wide or path-scoped.
///
/// Repository-wide ignores are `repo.config().doctor.ignore` unioned with
/// `extra_ignore` (e.g. CLI `--ignore` flags for a single invocation).
/// Path-scoped ignores are `repo.config().doctor.ignore_path`: each entry
/// suppresses its `rules` only for issues whose path, relative to the
/// repository root with separators normalized to `/`, matches its `glob`.
/// Both forms match rules case-insensitively against `Issue.rule_id` and
/// `Issue.rule_name` (issue #365).
///
/// Returns the filtered report, the count of issues that were suppressed
/// (repository-wide and path-scoped combined, each issue counted once even
/// if it matched both), and any config warnings: both `adrs.toml` and
/// `.adrs.toml` present, an invalid glob that could not be compiled, or a
/// `[[doctor.ignore_path]]` entry naming a rule that only ever produces
/// path-less diagnostics and so can never be suppressed this way.
pub fn check_all_filtered(
    repo: &Repository,
    extra_ignore: &[String],
) -> Result<(LintReport, usize, Vec<String>)> {
    let mut report = LintReport::new();
    let mut warnings = Vec::new();
    if repo.shadowed_toml().is_some() {
        warnings.push(crate::config::DUPLICATE_TOML_CONFIG_MESSAGE.to_string());
    }

    // Use list_with_errors to capture parse failures
    let (adrs, parse_errors) = repo.list_with_errors()?;

    // Report parse errors as lint issues
    for (path, error) in &parse_errors {
        report.add(Issue {
            rule_id: "parse-error".to_string(),
            rule_name: "adr-parse-error".to_string(),
            severity: IssueSeverity::Error,
            message: format!("Failed to parse ADR: {error}"),
            path: Some(path.clone()),
            line: None,
            column: None,
            adr_number: None,
            related_adrs: Vec::new(),
        });
    }

    // Run per-file lint on successfully parsed ADRs
    for adr in &adrs {
        let adr_report = lint_adr(adr)?;
        report.issues.extend(adr_report.issues);
    }

    // Run repository-level checks (these still use repo.list() internally,
    // which is fine — they only need successfully parsed ADRs)
    let repo_report = check_repository(repo)?;
    report.issues.extend(repo_report.issues);

    report.sort();

    let ignore_set: HashSet<String> = repo
        .config()
        .doctor
        .ignore
        .iter()
        .chain(extra_ignore.iter())
        .map(|s| s.to_lowercase())
        .collect();

    // Compile each `[[doctor.ignore_path]]` entry. An invalid glob is a
    // config error in the same family as #363 -- the user believes an
    // exemption is active when it is not -- so it is reported as a warning
    // rather than silently dropped, and the rest of the config still loads
    // and applies (see `warn_unknown_config_keys` in the CLI for the same
    // non-fatal-warning convention).
    let mut compiled_ignore_paths: Vec<CompiledIgnorePath> = Vec::new();
    for entry in &repo.config().doctor.ignore_path {
        match Glob::new(&entry.glob) {
            Ok(glob) => compiled_ignore_paths.push(CompiledIgnorePath {
                matcher: glob.compile_matcher(),
                rules: entry.rules.iter().map(|s| s.to_lowercase()).collect(),
            }),
            Err(e) => warnings.push(format!(
                "[[doctor.ignore_path]] glob '{}' is invalid and will not be applied: {e}",
                entry.glob
            )),
        }

        for rule in &entry.rules {
            if ALWAYS_PATHLESS_RULES
                .iter()
                .any(|pathless| pathless.eq_ignore_ascii_case(rule))
            {
                warnings.push(format!(
                    "[[doctor.ignore_path]] entry for glob '{}' names rule '{}', which only ever produces diagnostics without a path; this exemption can never suppress it",
                    entry.glob, rule
                ));
            }
        }
    }

    if ignore_set.is_empty() && compiled_ignore_paths.is_empty() {
        return Ok((report, 0, warnings));
    }

    let root = repo.root();
    let before = report.issues.len();
    report.issues.retain(|issue| {
        if ignore_set.contains(&issue.rule_id.to_lowercase())
            || ignore_set.contains(&issue.rule_name.to_lowercase())
        {
            return false;
        }

        let Some(path) = &issue.path else {
            return true;
        };

        let relative = path.strip_prefix(root).unwrap_or(path);
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        let rule_id = issue.rule_id.to_lowercase();
        let rule_name = issue.rule_name.to_lowercase();

        !compiled_ignore_paths.iter().any(|entry| {
            entry.matcher.is_match(&relative_str)
                && (entry.rules.contains(&rule_id) || entry.rules.contains(&rule_name))
        })
    });
    let suppressed = before - report.issues.len();

    Ok((report, suppressed, warnings))
}

/// Run all checks: per-file lint + repository-level checks.
///
/// Also reports files that look like ADRs (digit-prefixed `.md` files in the
/// ADR directory) but could not be parsed (e.g., invalid YAML frontmatter).
pub fn check_all(repo: &Repository) -> Result<LintReport> {
    check_all_filtered(repo, &[]).map(|(report, _, _)| report)
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
        // Uses the actual ADR #0001 text produced by `adrs init`. The word "described"
        // previously triggered an ADR014 false positive (fixed in mdbook-lint-rulesets 0.14.3).
        let content = format!(
            r#"# 1. Record architecture decisions

Date: 2024-03-04

## Status

Accepted

## Context

{}

## Decision

{}

## Consequences

{}
"#,
            crate::init_adr::CONTEXT,
            crate::init_adr::DECISION,
            crate::init_adr::CONSEQUENCES,
        );
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

    #[test]
    fn test_nygard_bare_minimal_template_passes_doctor() {
        // Regression for #330: a file produced by the Nygard bare-minimal
        // template must not trip any doctor error (it previously emitted no
        // `Date:` line and failed with ADR003). Empty-section ADR014 warnings
        // are inherent to the variant and are not errors.
        use crate::{Adr, Config, Repository, Template, TemplateFormat, TemplateVariant};

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let template =
            Template::builtin_with_variant(TemplateFormat::Nygard, TemplateVariant::BareMinimal);
        let adr = Adr::new(2, "Bare minimal regression");
        let rendered = template
            .render(&adr, &Config::default(), &std::collections::HashMap::new())
            .unwrap();
        let path = repo.adr_path().join("0002-bare-minimal-regression.md");
        std::fs::write(&path, rendered).unwrap();

        let report = check_all(&repo).unwrap();
        let file_errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .filter(|i| {
                i.path
                    .as_ref()
                    .is_some_and(|p| p.to_string_lossy().contains("0002-bare-minimal-regression"))
            })
            .collect();
        assert!(
            file_errors.is_empty(),
            "nygard bare-minimal output should have no doctor errors, got: {file_errors:?}"
        );
    }

    #[test]
    fn test_detect_adr_format_from_headings() {
        // Frontmatter + Nygard headings is Nygard, not MADR (#348).
        assert_eq!(
            detect_adr_format("---\nstatus: accepted\n---\n\n## Context\n\n## Decision\n"),
            AdrFormat::Nygard
        );
        // MADR-specific headings win regardless of frontmatter.
        assert_eq!(
            detect_adr_format(
                "---\nstatus: accepted\n---\n\n## Context and Problem Statement\n\n## Decision Outcome\n"
            ),
            AdrFormat::Madr4
        );
        // Plain Nygard (no frontmatter) is Nygard.
        assert_eq!(
            detect_adr_format("# 1. Title\n\nDate: 2024-03-04\n\n## Context\n"),
            AdrFormat::Nygard
        );
        // Neither heading set present: defer to the rules' own heuristic.
        assert_eq!(
            detect_adr_format("---\nstatus: accepted\n---\n\n# Title only\n"),
            AdrFormat::Auto
        );
    }

    #[test]
    fn test_ng_init_repo_passes_doctor() {
        // Regression for #348: `adrs --ng init` writes ADR #0001 with YAML
        // frontmatter and Nygard headings. The mdbook-lint ADR rules auto-detect
        // format from frontmatter alone and flagged it as MADR, demanding
        // `## Context and Problem Statement` / `## Decision Outcome` (ADR004/005).
        // A freshly initialized next-gen repository must pass doctor unchanged.
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let report = check_all(&repo).unwrap();
        let errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "freshly `--ng init`ed repo should pass doctor, got: {errors:?}"
        );
    }

    #[test]
    fn test_frontmatter_nygard_adr_not_flagged_for_madr_sections() {
        // #348: frontmatter presence alone must not trigger the MADR section
        // rules on a document that uses Nygard headings.
        let content = "---\nnumber: 1\ntitle: Record architecture decisions\ndate: 2024-03-04\nstatus: accepted\n---\n\n## Context\n\nSome context.\n\n## Decision\n\nSome decision.\n\n## Consequences\n\nSome consequences.\n";
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
        assert!(
            !report
                .issues
                .iter()
                .any(|i| i.rule_id == "ADR004" || i.rule_id == "ADR005"),
            "frontmatter+Nygard ADR must not trip MADR section rules, got: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_genuine_madr_missing_decision_outcome_still_flagged() {
        // Detection must not weaken validation of real MADR documents: a MADR
        // ADR (MADR headings) that omits `## Decision Outcome` still gets ADR005.
        let content = "---\nnumber: 1\ntitle: Use Postgres\ndate: 2024-03-04\nstatus: accepted\n---\n\n## Context and Problem Statement\n\nWhich database?\n\n## Considered Options\n\n* Postgres\n* MySQL\n";
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("adr").join("0001-use-postgres.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();

        let mut adr = Adr::new(1, "Use Postgres");
        adr.path = Some(path);

        let report = lint_adr(&adr).unwrap();
        assert!(
            report.issues.iter().any(|i| i.rule_id == "ADR005"),
            "MADR ADR missing '## Decision Outcome' must still trip ADR005, got: {:?}",
            report.issues
        );
    }

    #[test]
    fn test_check_all_reports_parse_errors() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        // Write an ADR with invalid YAML (bad date)
        let bad_content =
            "---\nnumber: 2\nstatus: accepted\ndate: not-a-date\n---\n\n# 2. Bad Date\n";
        std::fs::write(repo.adr_path().join("0002-bad-date.md"), bad_content).unwrap();

        let report = check_all(&repo).unwrap();

        let parse_errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "parse-error")
            .collect();

        assert_eq!(parse_errors.len(), 1, "should report 1 parse error");
        assert_eq!(parse_errors[0].severity, IssueSeverity::Error);
        assert!(
            parse_errors[0]
                .path
                .as_ref()
                .unwrap()
                .to_string_lossy()
                .contains("0002-bad-date.md")
        );
    }

    #[test]
    fn test_check_all_no_parse_errors_for_string_decision_makers() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        // Issue #216: decision-makers as string should not cause a parse error
        let content = "---\nnumber: 2\nstatus: accepted\ndate: 2026-03-18\ndecision-makers: alice\n---\n\n# 2. Test\n\n## Context\n\nContext.\n\n## Decision\n\nDecision.\n\n## Consequences\n\nConsequences.\n";
        std::fs::write(repo.adr_path().join("0002-test.md"), content).unwrap();

        let report = check_all(&repo).unwrap();

        let parse_errors: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "parse-error")
            .collect();

        assert!(
            parse_errors.is_empty(),
            "string decision-makers should not cause parse error, got: {:?}",
            parse_errors.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }
    // ========== check_repository collection rules (issue #239) ==========

    fn make_nygard_adr(number: u32, title: &str, status: &str, links: &str) -> String {
        format!(
            "# {}. {}\n\nDate: 2024-01-01\n\n## Status\n\n{}{}\n## Context\n\nSome context.\n\n## Decision\n\nA decision.\n\n## Consequences\n\nSome consequences.\n",
            number, title, status, links
        )
    }

    #[test]
    fn test_check_repository_broken_link_adr013() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // init creates ADR #1 automatically
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 2 links to nonexistent ADR 99
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(
                2,
                "Second",
                "Accepted",
                "\n\nSupersedes [99. Unknown](0099-unknown.md)\n",
            ),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        // Should have an ADR013 (broken links) issue
        let has_adr013 = report.issues.iter().any(|i| i.rule_id == "ADR013");
        assert!(
            has_adr013,
            "Expected ADR013 broken-link issue, got: {:?}",
            report.issues.iter().map(|i| &i.rule_id).collect::<Vec<_>>()
        );
    }

    fn make_frontmatter_adr(number: u32, title: &str, status: &str, links_yaml: &str) -> String {
        format!(
            "---\nnumber: {}\ntitle: {}\ndate: 2024-01-01\nstatus: {}\n{}---\n\n## Context\n\nSome context.\n\n## Decision\n\nA decision.\n\n## Consequences\n\nSome consequences.\n",
            number, title, status, links_yaml
        )
    }

    #[test]
    fn test_check_repository_frontmatter_broken_link_adr013_error() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // nextgen mode: links live in YAML frontmatter (issue #355 repro)
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 2 has a frontmatter link to nonexistent ADR 99
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_frontmatter_adr(
                2,
                "Second",
                "proposed",
                "links:\n  - target: 99\n    kind: relatesto\n",
            ),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        let broken = report
            .issues
            .iter()
            .find(|i| i.rule_id == "ADR013" && i.severity == IssueSeverity::Error);
        assert!(
            broken.is_some(),
            "Expected ADR013 error for frontmatter link to non-existent ADR 99, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, i.severity, &i.message))
                .collect::<Vec<_>>()
        );
        let issue = broken.unwrap();
        assert_eq!(issue.adr_number, Some(2));
        assert!(
            issue.path.is_some(),
            "expected a file location on the issue"
        );
        assert!(
            issue.message.contains("links to non-existent ADR 99"),
            "unexpected message: {}",
            issue.message
        );
        assert!(
            report.has_errors(),
            "a broken frontmatter link must make the report (and doctor's exit code) nonzero"
        );
    }

    #[test]
    fn test_check_repository_frontmatter_link_to_existing_adr_no_issue() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        // init() creates ADR #1; link ADR 2 to it.
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_frontmatter_adr(
                2,
                "Second",
                "proposed",
                "links:\n  - target: 1\n    kind: relatesto\n",
            ),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        assert!(
            !report.issues.iter().any(|i| i.rule_id == "ADR013"),
            "link to an existing ADR should not produce ADR013, got: {:?}",
            report.issues.iter().map(|i| &i.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_dedups_frontmatter_and_body_broken_link() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        // A Nygard-format nextgen render puts a broken link's target in both
        // the frontmatter `links:` block and a body markdown link, with the
        // same unresolved fallback filename (see template.rs's
        // `resolve_link_titles`, which falls back to `{:04}-....md` for a
        // target it can't find). One broken link should report once.
        let content = "---\nnumber: 2\ntitle: Second\ndate: 2024-01-01\nstatus: proposed\nlinks:\n  - target: 99\n    kind: relatesto\n---\n\n# 2. Second\n\nDate: 2024-01-01\n\n## Status\n\nProposed\n\nRelates to [99. ...](0099-....md)\n\n## Context\n\nSome context.\n\n## Decision\n\nA decision.\n\n## Consequences\n\nSome consequences.\n";
        std::fs::write(adr_dir.join("0002-second.md"), content).unwrap();

        let report = check_repository(&repo).unwrap();

        let adr013_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR013")
            .collect();
        assert_eq!(
            adr013_issues.len(),
            1,
            "a broken link present in both frontmatter and body should report once, got: {:?}",
            adr013_issues
                .iter()
                .map(|i| (&i.severity, &i.message))
                .collect::<Vec<_>>()
        );
        assert_eq!(adr013_issues[0].severity, IssueSeverity::Error);
    }

    #[test]
    fn test_check_repository_dedups_frontmatter_and_body_link_with_real_filename() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        // A link that was rendered while its target still existed carries the
        // target's real filename, not the `{:04}-....md` unresolved fallback.
        // This is the case #355 was found through: renumbering left a stale
        // target behind. It is still one broken link and must report once.
        let content = "---\nnumber: 2\ntitle: Second\ndate: 2024-01-01\nstatus: proposed\nlinks:\n  - target: 99\n    kind: relatesto\n---\n\n# 2. Second\n\nDate: 2024-01-01\n\n## Status\n\nProposed\n\nRelates to [99. Old Title](0099-old-title.md)\n\n## Context\n\nSome context.\n\n## Decision\n\nA decision.\n\n## Consequences\n\nSome consequences.\n";
        std::fs::write(adr_dir.join("0002-second.md"), content).unwrap();

        let report = check_repository(&repo).unwrap();

        let adr013_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR013")
            .collect();
        assert_eq!(
            adr013_issues.len(),
            1,
            "a broken link whose body filename resolved should report once, got: {:?}",
            adr013_issues
                .iter()
                .map(|i| (&i.severity, &i.message))
                .collect::<Vec<_>>()
        );
        assert_eq!(adr013_issues[0].severity, IssueSeverity::Error);
    }

    // ========== check_repository asymmetric-link rule (issue #357) ==========

    #[test]
    fn test_check_repository_asymmetric_link_half_deleted_one_warning() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // Compatible mode, matching the issue's own reproduction. `init`
        // creates ADR #1 with no links -- the "half-deleted" state: its
        // `Superseded by` line was removed by hand, leaving ADR #2's
        // `Supersedes` link with nothing pointing back (issue #357 case 1).
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(
                2,
                "Second",
                "Accepted",
                "\n\nSupersedes [1. Record architecture decisions](0001-record-architecture-decisions.md)\n",
            ),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "asymmetric-link")
            .collect();
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one asymmetric-link warning, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );
        assert_eq!(warnings[0].rule_name, "adr-asymmetric-link");
        assert_eq!(warnings[0].severity, IssueSeverity::Warning);
        assert_eq!(warnings[0].adr_number, Some(2));
        assert!(
            warnings[0].message.contains("ADR 2")
                && warnings[0].message.contains("ADR 1")
                && warnings[0].message.contains("no link back to ADR 2"),
            "unexpected message: {}",
            warnings[0].message
        );
    }

    #[test]
    fn test_check_repository_asymmetric_link_mis_targeted_two_warnings() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // Nextgen mode (frontmatter links) -- covers the both-modes requirement.
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 1's reverse link is repointed at ADR 3, which exists but has no
        // relationship to either. ADR 2 still claims to supersede ADR 1
        // (issue #357 case 2). Both halves are independently broken, so this
        // must produce two warnings, not one.
        std::fs::write(
            adr_dir.join("0001-record-architecture-decisions.md"),
            make_frontmatter_adr(
                1,
                "Record architecture decisions",
                "superseded",
                "links:\n  - target: 3\n    kind: supersededby\n",
            ),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_frontmatter_adr(
                2,
                "Second",
                "proposed",
                "links:\n  - target: 1\n    kind: supersedes\n",
            ),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0003-third.md"),
            make_frontmatter_adr(3, "Third", "proposed", ""),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "asymmetric-link")
            .collect();
        assert_eq!(
            warnings.len(),
            2,
            "expected two asymmetric-link warnings, one per independently broken half, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );

        let adr_numbers: Vec<_> = warnings.iter().filter_map(|i| i.adr_number).collect();
        assert!(
            adr_numbers.contains(&1) && adr_numbers.contains(&2),
            "expected warnings naming both ADR 1 and ADR 2, got: {adr_numbers:?}"
        );
        assert!(
            warnings
                .iter()
                .all(|i| i.severity == IssueSeverity::Warning)
        );
    }

    #[test]
    fn test_check_repository_symmetric_link_via_repository_link_no_warning() {
        use crate::{LinkKind, Repository};

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 2, so `repo.link` has two existing ADRs to connect.
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_frontmatter_adr(2, "Second", "proposed", ""),
        )
        .unwrap();

        // Built through the tool's own API, not by hand, so this test breaks
        // if the rule and `adrs link` ever disagree about what counts as
        // symmetric.
        repo.link(2, 1, LinkKind::Supersedes, LinkKind::SupersededBy)
            .unwrap();

        let report = check_repository(&repo).unwrap();

        assert!(
            !report.issues.iter().any(|i| i.rule_id == "asymmetric-link"),
            "a pair built through Repository::link must not be flagged, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_non_derived_reverse_kind_no_warning() {
        use crate::{LinkKind, Repository};

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        let adr_dir = repo.adr_path();

        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_frontmatter_adr(2, "Second", "proposed", ""),
        )
        .unwrap();

        // `adrs link` accepts an explicit `reverse_kind` override
        // (commands/link.rs), so a pair whose reverse kind is not
        // `LinkKind::Supersedes.reverse()` (`SupersededBy`) is still valid
        // tool output, not corruption. Here ADR 1's link back to ADR 2 is
        // `RelatesTo` rather than the derived `SupersededBy`.
        repo.link(2, 1, LinkKind::Supersedes, LinkKind::RelatesTo)
            .unwrap();

        let report = check_repository(&repo).unwrap();

        assert!(
            !report.issues.iter().any(|i| i.rule_id == "asymmetric-link"),
            "a non-derived but present reverse link must not be flagged, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_broken_link_no_asymmetric_link_warning() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // init creates ADR #1 automatically.
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 2 links to nonexistent ADR 99: a broken link, not an
        // asymmetric one. It must be reported once, as ADR013, and not
        // again as asymmetric-link.
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(
                2,
                "Second",
                "Accepted",
                "\n\nSupersedes [99. Unknown](0099-unknown.md)\n",
            ),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        assert!(
            report.issues.iter().any(|i| i.rule_id == "ADR013"),
            "expected the broken-link diagnostic to still fire"
        );
        assert!(
            !report.issues.iter().any(|i| i.rule_id == "asymmetric-link"),
            "a link to a nonexistent ADR must not also be flagged as asymmetric, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_symmetric_relates_to_no_warning() {
        use crate::{LinkKind, Repository};

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(2, "Second", "Accepted", ""),
        )
        .unwrap();

        // `LinkKind::RelatesTo.reverse()` is `RelatesTo` itself, so a
        // symmetric `RelatesTo` pair is the default `repo.link` output.
        repo.link(2, 1, LinkKind::RelatesTo, LinkKind::RelatesTo)
            .unwrap();

        let report = check_repository(&repo).unwrap();

        assert!(
            !report.issues.iter().any(|i| i.rule_id == "asymmetric-link"),
            "a symmetric RelatesTo pair must not be flagged, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_one_way_relates_to_one_warning() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // init creates ADR #1 with no links.
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 2 relates to ADR 1, but ADR 1 has no reciprocal link -- a
        // one-way RelatesTo is a plausible hand-written relationship, and
        // should be a single warning, not silence.
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(
                2,
                "Second",
                "Accepted",
                "\n\nRelates to [1. Record architecture decisions](0001-record-architecture-decisions.md)\n",
            ),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        let warnings: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "asymmetric-link")
            .collect();
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one asymmetric-link warning for a one-way RelatesTo, got: {:?}",
            report
                .issues
                .iter()
                .map(|i| (&i.rule_id, &i.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_sequential_gap_adr011() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // init creates ADR #1 automatically; write #2 and #4 to create a gap at #3
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // ADRs 1, 2, 4 -- gap at 3
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(2, "Second", "Accepted", ""),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0004-fourth.md"),
            make_nygard_adr(4, "Fourth", "Accepted", ""),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        // Should have an ADR011 (sequential gap) issue
        let has_adr011 = report.issues.iter().any(|i| i.rule_id == "ADR011");
        assert!(
            has_adr011,
            "Expected ADR011 sequential-gap issue, got: {:?}",
            report.issues.iter().map(|i| &i.rule_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_repository_clean_repo_has_no_issues() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // Repository::init creates ADR #1 automatically -- use #2 and #3 to avoid duplicate
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(2, "Second", "Accepted", ""),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0003-third.md"),
            make_nygard_adr(3, "Third", "Proposed", ""),
        )
        .unwrap();

        let report = check_repository(&repo).unwrap();

        let collection_rule_ids = ["ADR010", "ADR011", "ADR012", "ADR013"];
        let collection_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| collection_rule_ids.contains(&i.rule_id.as_str()))
            .collect();

        assert!(
            collection_issues.is_empty(),
            "Clean repo should have no collection-rule issues, got: {:?}",
            collection_issues
                .iter()
                .map(|i| format!("{}: {}", i.rule_id, i.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_check_all_combines_lint_and_repository_checks() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // Create a valid ADR so check_all has something to process
        std::fs::write(
            adr_dir.join("0001-first.md"),
            make_nygard_adr(1, "First", "Accepted", ""),
        )
        .unwrap();

        // check_all should succeed and return a report
        let report = check_all(&repo).unwrap();

        // With a valid sequential repo, no collection-rule violations
        let adr011 = report
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR011")
            .count();
        assert_eq!(
            adr011, 0,
            "Single valid ADR should have no sequential-gap issue"
        );
    }

    // ========== check_all_filtered / [doctor].ignore (issue #316) ==========

    #[test]
    fn test_check_all_filtered_suppresses_ignored_rule() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        // init creates ADR #1 automatically; write #2 and #4 to create a gap at #3,
        // which trips ADR011 (Warning severity, confirmed via
        // test_check_repository_sequential_gap_adr011).
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(2, "Second", "Accepted", ""),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0004-fourth.md"),
            make_nygard_adr(4, "Fourth", "Accepted", ""),
        )
        .unwrap();

        // Unfiltered: check_repository still reports ADR011.
        let unfiltered = check_repository(&repo).unwrap();
        let unfiltered_adr011 = unfiltered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR011")
            .count();
        assert!(
            unfiltered_adr011 > 0,
            "expected check_repository to report ADR011 before filtering"
        );

        // Write adrs.toml with a lowercase ignore entry, then re-open the repository
        // so the config is loaded from disk (Repository::init keeps the in-memory
        // config it built at creation time).
        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[doctor]\nignore = [\"adr011\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();
        assert_eq!(repo.config().doctor.ignore, vec!["adr011".to_string()]);

        // check_all (and check_all_filtered) should no longer contain ADR011,
        // proving case-insensitive matching against the real rule_id "ADR011".
        let filtered = check_all(&repo).unwrap();
        let filtered_adr011 = filtered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR011")
            .count();
        assert_eq!(
            filtered_adr011, 0,
            "check_all should suppress ADR011 issues per [doctor].ignore"
        );

        // check_repository (unfiltered) should still report ADR011 -- filtering
        // is check_all-level only.
        let still_unfiltered = check_repository(&repo).unwrap();
        assert!(
            still_unfiltered
                .issues
                .iter()
                .any(|i| i.rule_id == "ADR011"),
            "check_repository should remain unfiltered"
        );
    }

    #[test]
    fn test_check_all_filtered_returns_suppressed_count() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(2, "Second", "Accepted", ""),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0004-fourth.md"),
            make_nygard_adr(4, "Fourth", "Accepted", ""),
        )
        .unwrap();

        let unfiltered = check_all(&repo).unwrap();
        let unfiltered_adr011 = unfiltered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR011")
            .count();
        assert!(unfiltered_adr011 > 0);

        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[doctor]\nignore = [\"ADR011\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert_eq!(suppressed_count, unfiltered_adr011);
        assert!(
            filtered.issues.iter().all(|i| i.rule_id != "ADR011"),
            "filtered report should not contain ADR011"
        );
        assert!(
            warnings.is_empty(),
            "no [[doctor.ignore_path]] entries configured, expected no warnings"
        );
    }

    // ========== check_all_filtered / [[doctor.ignore_path]] (issue #365) ==========

    /// An ADR with an explicit Consequences body, so a test can trigger
    /// ADR014's placeholder-text check on demand.
    fn make_nygard_adr_with_consequences(
        number: u32,
        title: &str,
        status: &str,
        consequences: &str,
    ) -> String {
        format!(
            "# {number}. {title}\n\nDate: 2024-01-01\n\n## Status\n\n{status}\n\n## Context\n\nSome context.\n\n## Decision\n\nA decision.\n\n## Consequences\n\n{consequences}\n"
        )
    }

    #[test]
    fn test_check_all_filtered_scoped_ignore_suppresses_on_matching_record_only() {
        // The headline test (#365): a scoped ignore suppresses ADR014 on the
        // record it names and leaves ADR014 firing on a different record that
        // trips the same placeholder-text check.
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();

        // ADR 1 (created by init) already has real content; add two more ADRs
        // that both trip ADR014 via placeholder text in Consequences.
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr_with_consequences(2, "Second", "Accepted", "TBD"),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0003-third.md"),
            make_nygard_adr_with_consequences(3, "Third", "Accepted", "TBD"),
        )
        .unwrap();

        // Unfiltered: both records trip ADR014.
        let (unfiltered, _, _) = check_all_filtered(&repo, &[]).unwrap();
        let unfiltered_adr014_paths: Vec<_> = unfiltered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR014")
            .filter_map(|i| i.path.clone())
            .collect();
        assert!(
            unfiltered_adr014_paths
                .iter()
                .any(|p| p.ends_with("0002-second.md")),
            "expected ADR014 on 0002-second.md before scoping, got: {unfiltered_adr014_paths:?}"
        );
        assert!(
            unfiltered_adr014_paths
                .iter()
                .any(|p| p.ends_with("0003-third.md")),
            "expected ADR014 on 0003-third.md before scoping, got: {unfiltered_adr014_paths:?}"
        );

        // Scope the exemption to 0002-second.md only.
        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[[doctor.ignore_path]]\nglob = \"doc/adr/0002-*.md\"\nrules = [\"ADR014\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {warnings:?}"
        );
        assert!(suppressed_count > 0);

        let filtered_adr014_paths: Vec<_> = filtered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR014")
            .filter_map(|i| i.path.clone())
            .collect();
        assert!(
            !filtered_adr014_paths
                .iter()
                .any(|p| p.ends_with("0002-second.md")),
            "0002-second.md's ADR014 should be suppressed, got: {filtered_adr014_paths:?}"
        );
        assert!(
            filtered_adr014_paths
                .iter()
                .any(|p| p.ends_with("0003-third.md")),
            "0003-third.md's ADR014 should still fire, got: {filtered_adr014_paths:?}"
        );
    }

    #[test]
    fn test_check_all_filtered_scoped_ignore_double_star_matches_subdirectory() {
        // `**` must span the intermediate directory components of a nested
        // `adr_dir` (e.g. "docs/architecture/decisions", the shape used in
        // #363's own reproduction), not just match within a single directory.
        // `Repository::list` only reads ADR files directly inside `adr_dir`
        // (`max_depth(1)`), so the subdirectory being spanned here is
        // `adr_dir` itself relative to the repository root, not a
        // subdirectory of `adr_dir`.
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(
            temp.path(),
            Some(PathBuf::from("docs/architecture/decisions")),
            false,
        )
        .unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr_with_consequences(2, "Second", "Accepted", "TBD"),
        )
        .unwrap();

        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"docs/architecture/decisions\"\n\n[[doctor.ignore_path]]\nglob = \"**/0002-*.md\"\nrules = [\"ADR014\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {warnings:?}"
        );
        assert!(
            suppressed_count > 0,
            "expected the ** glob to match the nested record"
        );
        assert!(
            !filtered.issues.iter().any(|i| i.rule_id == "ADR014"
                && i.path
                    .as_ref()
                    .is_some_and(|p| p.ends_with("0002-second.md"))),
            "nested record's ADR014 should be suppressed by the ** glob"
        );
    }

    #[test]
    fn test_check_all_filtered_scoped_ignore_matching_nothing_suppresses_nothing() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr_with_consequences(2, "Second", "Accepted", "TBD"),
        )
        .unwrap();

        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[[doctor.ignore_path]]\nglob = \"doc/adr/9999-*.md\"\nrules = [\"ADR014\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {warnings:?}"
        );
        assert_eq!(
            suppressed_count, 0,
            "a glob matching nothing should suppress nothing"
        );
        assert!(
            filtered.issues.iter().any(|i| i.rule_id == "ADR014"),
            "ADR014 should still fire since the glob did not match"
        );
    }

    #[test]
    fn test_check_all_filtered_scoped_and_repo_wide_ignores_compose() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        // 0002 trips ADR014 (scoped away below); 0003/0005 create a numbering
        // gap that trips ADR011 (suppressed repository-wide below).
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr_with_consequences(2, "Second", "Accepted", "TBD"),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0003-third.md"),
            make_nygard_adr(3, "Third", "Accepted", ""),
        )
        .unwrap();
        std::fs::write(
            adr_dir.join("0005-fifth.md"),
            make_nygard_adr(5, "Fifth", "Accepted", ""),
        )
        .unwrap();

        let (unfiltered, _, _) =
            check_all_filtered(&Repository::open(temp.path()).unwrap(), &[]).unwrap();
        let unfiltered_adr014 = unfiltered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR014")
            .count();
        let unfiltered_adr011 = unfiltered
            .issues
            .iter()
            .filter(|i| i.rule_id == "ADR011")
            .count();
        assert!(unfiltered_adr014 > 0);
        assert!(unfiltered_adr011 > 0);

        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[doctor]\nignore = [\"ADR011\"]\n\n[[doctor.ignore_path]]\nglob = \"doc/adr/0002-*.md\"\nrules = [\"ADR014\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {warnings:?}"
        );
        assert_eq!(
            suppressed_count,
            unfiltered_adr014 + unfiltered_adr011,
            "both the scoped and repository-wide ignores should count, with no double counting"
        );
        assert!(!filtered.issues.iter().any(|i| i.rule_id == "ADR014"));
        assert!(!filtered.issues.iter().any(|i| i.rule_id == "ADR011"));
    }

    #[test]
    fn test_check_all_filtered_scoped_ignore_matches_by_rule_name() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr_with_consequences(2, "Second", "Accepted", "TBD"),
        )
        .unwrap();

        // Confirm ADR014's rule name before relying on it below.
        let (unfiltered, _, _) =
            check_all_filtered(&Repository::open(temp.path()).unwrap(), &[]).unwrap();
        let rule_name = unfiltered
            .issues
            .iter()
            .find(|i| i.rule_id == "ADR014")
            .map(|i| i.rule_name.clone())
            .expect("expected an ADR014 issue");

        std::fs::write(
            temp.path().join("adrs.toml"),
            format!(
                "adr_dir = \"doc/adr\"\n\n[[doctor.ignore_path]]\nglob = \"doc/adr/0002-*.md\"\nrules = [\"{rule_name}\"]\n"
            ),
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got {warnings:?}"
        );
        assert!(suppressed_count > 0);
        assert!(!filtered.issues.iter().any(|i| i.rule_id == "ADR014"));
    }

    #[test]
    fn test_check_all_filtered_invalid_glob_warns_and_config_still_loads() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr_with_consequences(2, "Second", "Accepted", "TBD"),
        )
        .unwrap();

        // '[' with no closing ']' is an invalid glob pattern.
        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[[doctor.ignore_path]]\nglob = \"doc/adr/[0002-*.md\"\nrules = [\"ADR014\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (filtered, suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert_eq!(
            suppressed_count, 0,
            "an invalid glob must not suppress anything"
        );
        assert!(
            filtered.issues.iter().any(|i| i.rule_id == "ADR014"),
            "ADR014 should still fire since the invalid glob was not applied"
        );
        assert!(
            warnings.iter().any(|w| w.contains("doc/adr/[0002-*.md")),
            "expected a warning naming the invalid glob, got: {warnings:?}"
        );
    }

    #[test]
    fn test_check_all_filtered_ignore_path_naming_collection_rule_warns() {
        use crate::Repository;

        let temp = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adr_dir = repo.adr_path();
        std::fs::write(
            adr_dir.join("0002-second.md"),
            make_nygard_adr(2, "Second", "Accepted", ""),
        )
        .unwrap();

        // ADR011 (sequential numbering) is an upstream collection rule and
        // never carries a path, so this exemption can never fire.
        std::fs::write(
            temp.path().join("adrs.toml"),
            "adr_dir = \"doc/adr\"\n\n[[doctor.ignore_path]]\nglob = \"doc/adr/0002-*.md\"\nrules = [\"ADR011\"]\n",
        )
        .unwrap();
        let repo = Repository::open(temp.path()).unwrap();

        let (_filtered, _suppressed_count, warnings) = check_all_filtered(&repo, &[]).unwrap();
        assert!(
            warnings.iter().any(|w| w.contains("ADR011")),
            "expected a warning naming the rule that can never fire, got: {warnings:?}"
        );
    }

    #[test]
    fn test_open_shadowed_toml_for_file_combinations() {
        use crate::{CONFIG_FILE, HIDDEN_CONFIG_FILE, LEGACY_CONFIG_FILE, Repository};

        for (toml, hidden, legacy, expect_shadow) in [
            (false, false, false, false),
            (false, false, true, false),
            (false, true, false, false),
            (false, true, true, false),
            (true, false, false, false),
            (true, false, true, false),
            (true, true, false, true),
            (true, true, true, true),
        ] {
            let temp = tempfile::tempdir().unwrap();
            Repository::init(temp.path(), None, false).unwrap();
            let _ = std::fs::remove_file(temp.path().join(CONFIG_FILE));
            let _ = std::fs::remove_file(temp.path().join(HIDDEN_CONFIG_FILE));
            let _ = std::fs::remove_file(temp.path().join(LEGACY_CONFIG_FILE));
            if toml {
                std::fs::write(temp.path().join(CONFIG_FILE), "adr_dir = \"doc/adr\"\n").unwrap();
            }
            if hidden {
                std::fs::write(
                    temp.path().join(HIDDEN_CONFIG_FILE),
                    "adr_dir = \"doc/adr\"\n",
                )
                .unwrap();
            }
            if legacy {
                std::fs::write(temp.path().join(LEGACY_CONFIG_FILE), "doc/adr\n").unwrap();
            }

            let repo = Repository::open(temp.path()).unwrap();
            assert_eq!(
                repo.shadowed_toml().is_some(),
                expect_shadow,
                "shadowed_toml mismatch for toml={toml} hidden={hidden} legacy={legacy}"
            );

            let (_, _, warnings) = check_all_filtered(&repo, &[]).unwrap();
            assert_eq!(
                warnings
                    .iter()
                    .any(|w| w == crate::config::DUPLICATE_TOML_CONFIG_MESSAGE),
                expect_shadow,
                "dual-TOML config warning mismatch for toml={toml} hidden={hidden} legacy={legacy}, got {warnings:?}"
            );
        }
    }
}
