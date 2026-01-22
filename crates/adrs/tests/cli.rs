//! CLI integration tests for the adrs binary.

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

/// Get a command for the adrs binary.
fn adrs() -> Command {
    Command::cargo_bin("adrs").unwrap()
}

// ============================================================================
// Help and Version
// ============================================================================

#[test]
fn test_help() {
    adrs()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Architecture Decision Records"));
}

#[test]
fn test_version() {
    adrs()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_subcommand_help_init() {
    adrs()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize a new ADR repository"));
}

#[test]
fn test_subcommand_help_new() {
    adrs()
        .args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new ADR"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--status"));
}

#[test]
fn test_subcommand_help_list() {
    adrs()
        .args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List all ADRs"));
}

// ============================================================================
// Init Command
// ============================================================================

#[test]
fn test_init_creates_directory_and_first_adr() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Check .adr-dir file was created
    temp.child(".adr-dir").assert(predicate::path::exists());
    temp.child(".adr-dir")
        .assert(predicate::str::contains("doc/adr"));

    // Check ADR directory was created
    temp.child("doc/adr").assert(predicate::path::is_dir());

    // Check initial ADR was created
    temp.child("doc/adr/0001-record-architecture-decisions.md")
        .assert(predicate::path::exists());

    temp.close().unwrap();
}

#[test]
fn test_init_with_custom_directory() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["init", "decisions"])
        .assert()
        .success();

    // Check custom directory was used
    temp.child(".adr-dir")
        .assert(predicate::str::contains("decisions"));
    temp.child("decisions").assert(predicate::path::is_dir());
    temp.child("decisions/0001-record-architecture-decisions.md")
        .assert(predicate::path::exists());

    temp.close().unwrap();
}

#[test]
fn test_init_with_cwd_flag() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .args(["-C", temp.path().to_str().unwrap(), "init"])
        .assert()
        .success();

    temp.child(".adr-dir").assert(predicate::path::exists());

    temp.close().unwrap();
}

// ============================================================================
// List Command
// ============================================================================

#[test]
fn test_list_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .failure();

    temp.close().unwrap();
}

#[test]
fn test_list_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // List outputs file paths
    adrs()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0001-record-architecture-decisions.md",
        ));

    temp.close().unwrap();
}

// ============================================================================
// New Command (non-interactive tests)
// ============================================================================

#[test]
fn test_new_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Test ADR"])
        .env("EDITOR", "true") // Use 'true' as a no-op editor
        .assert()
        .failure()
        .stderr(predicate::str::contains("No ADR repository found"));

    temp.close().unwrap();
}

// ============================================================================
// Config Command
// ============================================================================

#[test]
fn test_config_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Config shows defaults even without init (graceful behavior)
    adrs()
        .current_dir(temp.path())
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config source: defaults"));

    temp.close().unwrap();
}

#[test]
fn test_config_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Config should show the directory
    adrs()
        .current_dir(temp.path())
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("doc/adr"));

    temp.close().unwrap();
}

// ============================================================================
// Generate Commands
// ============================================================================

#[test]
fn test_generate_toc_help() {
    adrs()
        .args(["generate", "toc", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate a table of contents"));
}

#[test]
fn test_generate_toc_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Generate TOC
    adrs()
        .current_dir(temp.path())
        .args(["generate", "toc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Record architecture decisions"));

    temp.close().unwrap();
}

#[test]
fn test_generate_toc_ordered() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Generate ordered TOC
    adrs()
        .current_dir(temp.path())
        .args(["generate", "toc", "--ordered"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1."));

    temp.close().unwrap();
}

#[test]
fn test_generate_graph_help() {
    adrs()
        .args(["generate", "graph", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate a Graphviz graph"));
}

#[test]
fn test_generate_graph_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Generate graph
    adrs()
        .current_dir(temp.path())
        .args(["generate", "graph"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph"));

    temp.close().unwrap();
}

// ============================================================================
// Link Command
// ============================================================================

#[test]
fn test_link_help() {
    adrs()
        .args(["link", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Link two ADRs together"));
}

// ============================================================================
// Format Flag Tests
// ============================================================================

#[test]
fn test_new_format_flag_help() {
    adrs()
        .args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-f, --format"))
        .stdout(predicate::str::contains("nygard"));
}

// ============================================================================
// Global Flags
// ============================================================================

#[test]
fn test_ng_flag() {
    adrs()
        .args(["--ng", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NextGen mode"));
}

#[test]
fn test_cwd_flag() {
    adrs()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-C, --cwd"));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_unknown_command() {
    adrs()
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_missing_required_arg() {
    adrs()
        .arg("new")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================================================
// File Content Verification
// ============================================================================

#[test]
fn test_init_adr_content() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    let content = fs::read_to_string(
        temp.path()
            .join("doc/adr/0001-record-architecture-decisions.md"),
    )
    .unwrap();

    // Check essential sections
    assert!(content.contains("# 1. Record architecture decisions"));
    assert!(content.contains("## Status"));
    assert!(content.contains("## Context"));
    assert!(content.contains("## Decision"));
    assert!(content.contains("## Consequences"));

    temp.close().unwrap();
}

#[test]
fn test_adr_dir_file_content() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["init", "my/custom/path"])
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join(".adr-dir")).unwrap();
    assert_eq!(content.trim(), "my/custom/path");

    temp.close().unwrap();
}
