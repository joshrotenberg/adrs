//! Compatibility tests with npryce/adr-tools format.
//!
//! These tests verify that adrs-core can correctly parse and generate
//! ADRs in the format produced by the original adr-tools.

use adrs_core::{Adr, AdrStatus, Config, LinkKind, Parser, Repository};
use std::fs;
use tempfile::TempDir;

/// Sample ADR from adr-tools: 0001-record-architecture-decisions.md
const ADR_TOOLS_SAMPLE_1: &str = r#"# 1. Record architecture decisions

Date: 2016-02-12

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as described by Michael Nygard in this article: http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions

## Consequences

See Michael Nygard's article, linked above.
"#;

/// Sample ADR from adr-tools: 0002-implement-as-shell-scripts.md
const ADR_TOOLS_SAMPLE_2: &str = r#"# 2. Implement as shell scripts

Date: 2016-02-12

## Status

Accepted

## Context

ADRs are plain text files stored in a subdirectory of the project.
The tool needs to create and update that directory and the files in it.

It must work on a developer's machine, which could be running any common OS.

We want to get a working tool as quickly as possible.

## Decision

We will write the tool as shell scripts that run on any POSIX
compliant operating system.

## Consequences

We will only be able to use commands that are installed on a
vanilla development machine.
"#;

/// Sample ADR with more complex structure: 0003-single-command-with-subcommands.md
const ADR_TOOLS_SAMPLE_3: &str = r#"# 3. Single command with subcommands

Date: 2016-02-12

## Status

Accepted

## Context

The tool provides a number of related commands to create
and manipulate architecture decision records.

How can the user find out about the commands that are available?

## Decision

The tool defines a single command, called `adr`.

The first argument to `adr` (the subcommand) specifies the
action to perform.  Further arguments are interpreted by the
subcommand.

Running `adr` without any arguments lists the available
subcommands.

Subcommands are implemented as scripts in the same
directory as the `adr` script.  E.g. the subcommand `new` is
implemented as the script `adr-new`, the subcommand `help`
as the script `adr-help` and so on.

Helper scripts that are part of the implementation but not
subcommands follow a different naming convention, so that
subcommands can be listed by filtering and transforming script
file names.

## Consequences

Users can more easily explore the capabilities of the tool.

Users are already used to this style of command-line tool.  For
example, Git works this way.

Each subcommand can be implemented in the most appropriate
language.
"#;

/// Superseded ADR format (from adr-tools test expectations)
const ADR_TOOLS_SUPERSEDED: &str = r#"# 1. First Record

Date: 1992-01-12

## Status

Superceded by [2. Second Record](0002-second-record.md)

## Context

First context.

## Decision

First decision.

## Consequences

First consequences.
"#;

/// Superseding ADR format (from adr-tools test expectations)
const ADR_TOOLS_SUPERSEDING: &str = r#"# 2. Second Record

Date: 1992-01-12

## Status

Accepted

Supercedes [1. First Record](0001-first-record.md)

## Context

Second context.

## Decision

Second decision.

## Consequences

Second consequences.
"#;

// ========== Parsing Tests ==========

#[test]
fn test_parse_adr_tools_sample_1() {
    let parser = Parser::new();
    let adr = parser.parse(ADR_TOOLS_SAMPLE_1).unwrap();

    assert_eq!(adr.number, 1);
    assert_eq!(adr.title, "Record architecture decisions");
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert!(adr.context.contains("record the architectural decisions"));
    assert!(adr.decision.contains("Architecture Decision Records"));
    assert!(adr.consequences.contains("Michael Nygard"));
}

#[test]
fn test_parse_adr_tools_sample_2() {
    let parser = Parser::new();
    let adr = parser.parse(ADR_TOOLS_SAMPLE_2).unwrap();

    assert_eq!(adr.number, 2);
    assert_eq!(adr.title, "Implement as shell scripts");
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert!(adr.context.contains("plain text files"));
    assert!(adr.decision.contains("shell scripts"));
}

#[test]
fn test_parse_adr_tools_sample_3() {
    let parser = Parser::new();
    let adr = parser.parse(ADR_TOOLS_SAMPLE_3).unwrap();

    assert_eq!(adr.number, 3);
    assert_eq!(adr.title, "Single command with subcommands");
    assert_eq!(adr.status, AdrStatus::Accepted);

    // Check multiline content is preserved
    assert!(adr.context.contains("related commands"));
    assert!(adr.decision.contains("adr-new"));
    assert!(adr.consequences.contains("Git works this way"));
}

#[test]
fn test_parse_adr_tools_superseded() {
    let parser = Parser::new();
    let adr = parser.parse(ADR_TOOLS_SUPERSEDED).unwrap();

    assert_eq!(adr.number, 1);
    assert_eq!(adr.title, "First Record");
    // Note: adr-tools uses "Superceded" (typo), we treat it as Superseded
    assert_eq!(adr.status, AdrStatus::Superseded);
    assert_eq!(adr.links.len(), 1);
    assert_eq!(adr.links[0].target, 2);
    assert_eq!(adr.links[0].kind, LinkKind::SupersededBy);
}

#[test]
fn test_parse_adr_tools_superseding() {
    let parser = Parser::new();
    let adr = parser.parse(ADR_TOOLS_SUPERSEDING).unwrap();

    assert_eq!(adr.number, 2);
    assert_eq!(adr.title, "Second Record");
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert_eq!(adr.links.len(), 1);
    assert_eq!(adr.links[0].target, 1);
    assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
}

// ========== File Parsing Tests ==========

#[test]
fn test_parse_adr_tools_files_from_disk() {
    let parser = Parser::new();

    // These files are from the actual adr-tools repo
    let adr_tools_path = std::path::Path::new(".tmp/adr-tools/doc/adr");

    if !adr_tools_path.exists() {
        // Skip if adr-tools not available
        return;
    }

    // Parse all ADRs from adr-tools
    for entry in fs::read_dir(adr_tools_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map(|e| e == "md").unwrap_or(false) {
            let adr = parser.parse_file(&path).unwrap();

            // All should have valid numbers
            assert!(adr.number > 0, "ADR should have a number: {:?}", path);

            // All should have non-empty titles
            assert!(!adr.title.is_empty(), "ADR should have a title: {:?}", path);

            // All should have a valid status
            assert!(
                matches!(
                    adr.status,
                    AdrStatus::Proposed
                        | AdrStatus::Accepted
                        | AdrStatus::Deprecated
                        | AdrStatus::Superseded
                        | AdrStatus::Custom(_)
                ),
                "ADR should have a valid status: {:?}",
                path
            );
        }
    }
}

// ========== Repository Compatibility Tests ==========

#[test]
fn test_repository_creates_adr_tools_compatible_files() {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();

    // Create a new ADR
    let (_, path) = repo.new_adr("Use PostgreSQL").unwrap();
    let content = fs::read_to_string(&path).unwrap();

    // Should follow adr-tools format
    assert!(content.starts_with("# 2. Use PostgreSQL"));
    assert!(content.contains("Date:"));
    assert!(content.contains("## Status"));
    assert!(content.contains("## Context"));
    assert!(content.contains("## Decision"));
    assert!(content.contains("## Consequences"));

    // Should NOT have frontmatter in compatible mode
    assert!(!content.starts_with("---"));
}

#[test]
fn test_repository_supersede_creates_adr_tools_format() {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();

    // Supersede the initial ADR
    repo.supersede("New approach", 1).unwrap();

    // Check the superseded ADR
    let old_content = fs::read_to_string(
        repo.adr_path()
            .join("0001-record-architecture-decisions.md"),
    )
    .unwrap();
    assert!(old_content.contains("Superseded by"));
    assert!(old_content.contains("[2."));
    assert!(old_content.contains("0002-"));

    // Check the superseding ADR
    let new_content = fs::read_to_string(repo.adr_path().join("0002-new-approach.md")).unwrap();
    assert!(new_content.contains("Supersedes"));
    assert!(new_content.contains("[1."));
    assert!(new_content.contains("0001-"));
}

#[test]
fn test_roundtrip_adr_tools_format() {
    let temp = TempDir::new().unwrap();
    let adr_path = temp.path().join("doc/adr");
    fs::create_dir_all(&adr_path).unwrap();

    // Write an adr-tools format file
    fs::write(adr_path.join("0001-test-decision.md"), ADR_TOOLS_SAMPLE_1).unwrap();

    // Write the .adr-dir config
    fs::write(temp.path().join(".adr-dir"), "doc/adr").unwrap();

    // Open and read with our library
    let repo = Repository::open(temp.path()).unwrap();
    let adrs = repo.list().unwrap();

    assert_eq!(adrs.len(), 1);
    assert_eq!(adrs[0].number, 1);
    assert_eq!(adrs[0].title, "Record architecture decisions");
    assert_eq!(adrs[0].status, AdrStatus::Accepted);
}

#[test]
fn test_config_file_compatibility() {
    let temp = TempDir::new().unwrap();

    // Create .adr-dir file like adr-tools does
    fs::create_dir_all(temp.path().join("custom/path")).unwrap();
    fs::write(temp.path().join(".adr-dir"), "custom/path\n").unwrap();

    let config = Config::load(temp.path()).unwrap();
    assert_eq!(config.adr_dir, std::path::PathBuf::from("custom/path"));
}

#[test]
fn test_filename_format_compatibility() {
    // adr-tools uses: NNNN-slug.md format
    let adr = Adr::new(1, "Use Rust for Implementation");
    assert_eq!(adr.filename(), "0001-use-rust-for-implementation.md");

    let adr = Adr::new(42, "API v2.0 Design");
    assert_eq!(adr.filename(), "0042-api-v2-0-design.md");

    let adr = Adr::new(999, "Final Decision");
    assert_eq!(adr.filename(), "0999-final-decision.md");

    let adr = Adr::new(9999, "Max Four Digits");
    assert_eq!(adr.filename(), "9999-max-four-digits.md");
}

// ========== Edge Cases from adr-tools tests ==========

#[test]
fn test_funny_characters_in_title() {
    // From adr-tools test: funny-characters.sh
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();

    // Test various special characters
    let (adr, path) = repo.new_adr("Use a \"Alarm Clock\" for alerts").unwrap();
    assert!(path.exists());
    assert_eq!(adr.title, "Use a \"Alarm Clock\" for alerts");

    let (adr, path) = repo.new_adr("Use the 'Strategy' pattern").unwrap();
    assert!(path.exists());
    assert_eq!(adr.title, "Use the 'Strategy' pattern");
}

#[test]
fn test_alternative_adr_directory() {
    // From adr-tools test: alternative-adr-directory.sh
    let temp = TempDir::new().unwrap();

    // Use a non-default directory
    let _repo = Repository::init(temp.path(), Some("decisions".into()), false).unwrap();

    assert!(temp.path().join("decisions").exists());

    // .adr-dir should contain the custom path
    let config_content = fs::read_to_string(temp.path().join(".adr-dir")).unwrap();
    assert_eq!(config_content, "decisions");
}

#[test]
fn test_create_multiple_records() {
    // From adr-tools test: create-multiple-records.sh
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();

    // Create multiple ADRs
    let _ = repo.new_adr("First ADR").unwrap();
    let _ = repo.new_adr("Second ADR").unwrap();
    let _ = repo.new_adr("Third ADR").unwrap();

    let adrs = repo.list().unwrap();
    assert_eq!(adrs.len(), 4); // Including the init ADR

    // Check numbering
    assert_eq!(adrs[0].number, 1);
    assert_eq!(adrs[1].number, 2);
    assert_eq!(adrs[2].number, 3);
    assert_eq!(adrs[3].number, 4);
}

// ========== Status Parsing Compatibility ==========

#[test]
fn test_parse_various_status_formats() {
    let parser = Parser::new();

    // Standard statuses
    for (status_text, expected) in [
        ("Proposed", AdrStatus::Proposed),
        ("Accepted", AdrStatus::Accepted),
        ("Deprecated", AdrStatus::Deprecated),
        ("Superseded", AdrStatus::Superseded),
        // adr-tools typo variant
        ("Superceded", AdrStatus::Superseded),
    ] {
        let content = format!(
            r#"# 1. Test

## Status

{status_text}

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#
        );

        let adr = parser.parse(&content).unwrap();
        assert_eq!(
            adr.status, expected,
            "Failed to parse status: {status_text}"
        );
    }
}
