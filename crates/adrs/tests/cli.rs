//! CLI integration tests for the adrs binary.

use assert_cmd::{Command, cargo_bin_cmd};
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

/// Get a command for the adrs binary.
fn adrs() -> Command {
    cargo_bin_cmd!("adrs")
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
fn test_long_help_contains_env_vars() {
    adrs()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ENVIRONMENT VARIABLES"))
        .stdout(predicate::str::contains("ADR_DIRECTORY"));
}

#[test]
fn test_short_help_no_env_vars() {
    adrs()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("ENVIRONMENT VARIABLES").not());
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
// Status Command
// ============================================================================

#[test]
fn test_status_help() {
    adrs()
        .args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Change an ADR's status"));
}

#[test]
fn test_status_change_to_accepted() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize and create an ADR
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Test decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Change status to accepted
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "accepted"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to Accepted"));

    // Verify the file was updated
    let adr_path = temp.child("doc/adr/0002-test-decision.md");
    let content = fs::read_to_string(adr_path.path()).unwrap();
    assert!(content.contains("Accepted"));

    temp.close().unwrap();
}

#[test]
fn test_status_change_to_deprecated() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Old decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "deprecated"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to Deprecated"));

    temp.close().unwrap();
}

#[test]
fn test_status_superseded_with_by_flag() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create two ADRs
    adrs()
        .current_dir(temp.path())
        .args(["new", "Old decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "New decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Mark ADR 2 as superseded by ADR 3
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "superseded", "--by", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("superseded by 3"));

    // Verify the superseded-by link was added
    let adr_path = temp.child("doc/adr/0002-old-decision.md");
    let content = fs::read_to_string(adr_path.path()).unwrap();
    assert!(content.contains("Superseded"));
    assert!(content.contains("Superseded by"));

    temp.close().unwrap();
}

#[test]
fn test_status_custom_value() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Draft decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Set custom status
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "draft"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to draft"));

    temp.close().unwrap();
}

#[test]
fn test_status_adr_not_found() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Try to change status of non-existent ADR
    adrs()
        .current_dir(temp.path())
        .args(["status", "99", "accepted"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Failed")));

    temp.close().unwrap();
}

#[test]
fn test_status_by_flag_requires_superseded() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Test"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Try to use --by with accepted status (should fail)
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "accepted", "--by", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--by can only be used with 'superseded'",
        ));

    temp.close().unwrap();
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
    assert!(content.contains(
        "[Documenting Architecture Decisions](https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions)"
    ));
    assert!(content.contains("[adrs](https://github.com/joshrotenberg/adrs)"));
    assert!(content.contains("[adr-tools](https://github.com/npryce/adr-tools)"));

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

// ============================================================================
// Search Command
// ============================================================================

#[test]
fn test_search_help() {
    adrs()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search ADRs for matching content"));
}

#[test]
fn test_search_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["search", "test"])
        .assert()
        .failure();

    temp.close().unwrap();
}

#[test]
fn test_search_finds_matching_adr() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Search for text in the default ADR
    adrs()
        .current_dir(temp.path())
        .args(["search", "architecture"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Record architecture decisions"));

    temp.close().unwrap();
}

#[test]
fn test_search_title_only() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Search titles only - "architecture" is in the title
    adrs()
        .current_dir(temp.path())
        .args(["search", "-t", "architecture"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Record architecture decisions"));

    temp.close().unwrap();
}

#[test]
fn test_search_no_results() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Search for non-existent text
    adrs()
        .current_dir(temp.path())
        .args(["search", "xyzzyzzy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));

    temp.close().unwrap();
}

// ============================================================================
// Template Command
// ============================================================================

#[test]
fn test_template_help() {
    adrs()
        .args(["template", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage ADR templates"));
}

#[test]
fn test_template_list() {
    adrs()
        .args(["template", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nygard"))
        .stdout(predicate::str::contains("madr"));
}

#[test]
fn test_template_show_nygard() {
    adrs()
        .args(["template", "show", "nygard"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Status"))
        .stdout(predicate::str::contains("## Context"))
        .stdout(predicate::str::contains("## Decision"))
        .stdout(predicate::str::contains("## Consequences"));
}

#[test]
fn test_template_show_madr() {
    adrs()
        .args(["template", "show", "madr"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status"))
        .stdout(predicate::str::contains("Context"));
}

#[test]
fn test_template_show_minimal_variant() {
    adrs()
        .args(["template", "show", "nygard", "--variant", "minimal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Status"));
}

// ============================================================================
// Completions Command
// ============================================================================

#[test]
fn test_completions_help() {
    adrs()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate shell completions"))
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("zsh"))
        .stdout(predicate::str::contains("fish"));
}

#[test]
fn test_completions_bash() {
    adrs()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("adrs"));
}

#[test]
fn test_completions_zsh() {
    adrs()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef adrs"));
}

#[test]
fn test_completions_fish() {
    adrs()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

// ============================================================================
// Cheatsheet Command
// ============================================================================

#[test]
fn test_cheatsheet_help() {
    adrs()
        .args(["cheatsheet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show quick reference"));
}

#[test]
fn test_cheatsheet_output() {
    adrs()
        .arg("cheatsheet")
        .assert()
        .success()
        .stdout(predicate::str::contains("GETTING STARTED"))
        .stdout(predicate::str::contains("adrs init"))
        .stdout(predicate::str::contains("adrs new"));
}

// ============================================================================
// Export Command
// ============================================================================

#[test]
fn test_export_help() {
    adrs()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Export ADRs"));
}

#[test]
fn test_export_json_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"adrs\""))
        .stdout(predicate::str::contains("Record architecture decisions"));

    temp.close().unwrap();
}

// Tests for export.base_url config support (#300)

#[test]
fn test_export_json_config_base_url_no_flag() {
    // When export.base_url is set in adrs.toml, adrs export json (no --base-url)
    // should include source_uri fields.
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Write adrs.toml with [export] base_url
    fs::write(
        temp.path().join("adrs.toml"),
        "adr_dir = \"doc/adr\"\n\n[export]\nbase_url = \"https://example.com/adr\"\n",
    )
    .unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("source_uri"))
        .stdout(predicate::str::contains("https://example.com/adr"));

    temp.close().unwrap();
}

#[test]
fn test_export_json_flag_overrides_config_base_url() {
    // --base-url CLI flag takes precedence over export.base_url in config.
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        temp.path().join("adrs.toml"),
        "adr_dir = \"doc/adr\"\n\n[export]\nbase_url = \"https://config.example.com/adr\"\n",
    )
    .unwrap();

    adrs()
        .current_dir(temp.path())
        .args([
            "export",
            "json",
            "--base-url",
            "https://flag.example.com/adr",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://flag.example.com/adr"))
        .stdout(predicate::str::is_match("source_uri").unwrap());

    // The config URL should NOT appear
    let output = adrs()
        .current_dir(temp.path())
        .args([
            "export",
            "json",
            "--base-url",
            "https://flag.example.com/adr",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_str = String::from_utf8(output).unwrap();
    assert!(
        !output_str.contains("https://config.example.com/adr"),
        "config base_url should not appear when --base-url flag is set"
    );

    temp.close().unwrap();
}

#[test]
fn test_export_json_no_base_url_no_source_uri() {
    // When neither --base-url nor export.base_url in config is set,
    // export output should not contain source_uri fields.
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Write adrs.toml WITHOUT [export] section
    fs::write(temp.path().join("adrs.toml"), "adr_dir = \"doc/adr\"\n").unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("source_uri").not());

    temp.close().unwrap();
}

// ============================================================================
// Import Command
// ============================================================================

#[test]
fn test_import_help() {
    adrs()
        .args(["import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Import ADRs"));
}

#[test]
fn test_import_export_round_trip() {
    // Create a source repo with ADRs, export to JSON, import into fresh dir
    let src = assert_fs::TempDir::new().unwrap();

    // Initialize and create ADRs
    adrs()
        .current_dir(src.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(src.path())
        .args(["new", "Use PostgreSQL"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Export to JSON (stdout)
    let export_out = adrs()
        .current_dir(src.path())
        .args(["export", "json", "--pretty"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Write JSON to a temp file
    let json_file = src.child("export.json");
    fs::write(json_file.path(), &export_out).unwrap();

    // Import into a fresh directory
    let dest = assert_fs::TempDir::new().unwrap();
    let dest_adr_dir = dest.child("imported");

    adrs()
        .current_dir(dest.path())
        .args([
            "import",
            "json",
            "--dir",
            dest_adr_dir.path().to_str().unwrap(),
            json_file.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify ADRs were imported
    dest_adr_dir
        .child("0001-record-architecture-decisions.md")
        .assert(predicate::path::exists());
    dest_adr_dir
        .child("0002-use-postgresql.md")
        .assert(predicate::path::exists());

    src.close().unwrap();
    dest.close().unwrap();
}

#[test]
fn test_import_with_ng_flag() {
    let src = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(src.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(src.path())
        .args(["new", "NG Import Test"])
        .env("EDITOR", "true")
        .assert()
        .success();

    let export_out = adrs()
        .current_dir(src.path())
        .args(["export", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_file = src.child("export.json");
    fs::write(json_file.path(), &export_out).unwrap();

    let dest = assert_fs::TempDir::new().unwrap();
    let dest_adr_dir = dest.child("ng_imported");

    adrs()
        .current_dir(dest.path())
        .args([
            "import",
            "json",
            "--ng",
            "--dir",
            dest_adr_dir.path().to_str().unwrap(),
            json_file.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify the imported files have YAML frontmatter (ng_mode)
    let imported_path = dest_adr_dir
        .path()
        .join("0001-record-architecture-decisions.md");
    let content = fs::read_to_string(&imported_path).unwrap();
    assert!(
        content.starts_with("---"),
        "ng import should produce YAML frontmatter. Got:\n{content}"
    );

    src.close().unwrap();
    dest.close().unwrap();
}

#[test]
fn test_import_missing_file_exits_nonzero() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["import", "json", "/nonexistent/path/adrs.json"])
        .assert()
        .failure();

    temp.close().unwrap();
}

// ============================================================================
// List Filtering
// ============================================================================

#[test]
fn test_list_filter_by_status() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create a new ADR (starts as Proposed)
    adrs()
        .current_dir(temp.path())
        .args(["new", "Test decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // List only proposed ADRs - should only show ADR 2
    // (ADR 1 from init is Accepted by default)
    adrs()
        .current_dir(temp.path())
        .args(["list", "--status", "proposed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0002-test-decision.md"))
        .stdout(predicate::str::contains("0001-record-architecture").not());

    temp.close().unwrap();
}

// ============================================================================
// New Command with --no-edit flag
// ============================================================================

#[test]
fn test_new_no_edit_flag() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create ADR without editor
    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "Quick decision"])
        .assert()
        .success();

    // Verify ADR was created
    temp.child("doc/adr/0002-quick-decision.md")
        .assert(predicate::path::exists());

    temp.close().unwrap();
}

// ============================================================================
// Custom Template Flag Tests
// ============================================================================

#[test]
fn test_new_template_flag_missing_file() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Passing a non-existent template file should fail with a clear error
    adrs()
        .current_dir(temp.path())
        .args([
            "new",
            "--no-edit",
            "--template",
            "/nonexistent/path/template.md",
            "Should fail",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("custom template")
                .or(predicate::str::contains("/nonexistent/path/template.md")),
        );

    temp.close().unwrap();
}

// ============================================================================
// MADR Format Tests
// ============================================================================

#[test]
fn test_new_madr_format_listed() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create MADR format ADR
    adrs()
        .current_dir(temp.path())
        .args([
            "new",
            "--no-edit",
            "--format",
            "madr",
            "Use Redis for caching",
        ])
        .assert()
        .success();

    // Verify ADR appears in list
    adrs()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("0002-use-redis-for-caching.md"));

    temp.close().unwrap();
}

#[test]
fn test_new_madr_format_searchable() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create MADR format ADR
    adrs()
        .current_dir(temp.path())
        .args([
            "new",
            "--no-edit",
            "--format",
            "madr",
            "Use Redis for caching",
        ])
        .assert()
        .success();

    // Verify ADR is searchable by title
    adrs()
        .current_dir(temp.path())
        .args(["search", "Redis"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use Redis for caching"));

    temp.close().unwrap();
}

#[test]
fn test_madr_status_change() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create MADR format ADR
    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "--format", "madr", "Use GraphQL"])
        .assert()
        .success();

    // Change status
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "accepted"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to Accepted"));

    temp.close().unwrap();
}

#[test]
fn test_madr_export_includes_adr() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create MADR format ADR
    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "--format", "madr", "Use PostgreSQL"])
        .assert()
        .success();

    // Export should include the MADR ADR
    adrs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use PostgreSQL"));

    temp.close().unwrap();
}

// ============================================================================
// Smoke Tests - End-to-end workflow validation
// ============================================================================

/// Comprehensive smoke test simulating a real workflow with multiple ADRs
#[test]
fn test_smoke_full_workflow() {
    let temp = assert_fs::TempDir::new().unwrap();

    // 1. Initialize repository
    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // 2. Create Nygard format ADRs
    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "Use PostgreSQL for persistence"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "Implement REST API"])
        .assert()
        .success();

    // 3. Create MADR format ADR (verifies MADR parsing fix)
    adrs()
        .current_dir(temp.path())
        .args([
            "--ng",
            "new",
            "--no-edit",
            "--format",
            "madr",
            "Use Kubernetes for orchestration",
        ])
        .assert()
        .success();

    // 4. Verify all ADRs appear in list
    adrs()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0001-record-architecture-decisions.md",
        ))
        .stdout(predicate::str::contains(
            "0002-use-postgresql-for-persistence.md",
        ))
        .stdout(predicate::str::contains("0003-implement-rest-api.md"))
        .stdout(predicate::str::contains(
            "0004-use-kubernetes-for-orchestration.md",
        ));

    // 5. Search works for both formats
    adrs()
        .current_dir(temp.path())
        .args(["search", "PostgreSQL"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use PostgreSQL"));

    adrs()
        .current_dir(temp.path())
        .args(["search", "Kubernetes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use Kubernetes"));

    // 6. Change status
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "accepted"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to Accepted"));

    // 7. Create superseding ADR and link
    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "Use MySQL instead"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["link", "5", "Supersedes", "2"])
        .assert()
        .success();

    // 8. Export to JSON
    adrs()
        .current_dir(temp.path())
        .args(["export", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"adrs\""))
        .stdout(predicate::str::contains("Use PostgreSQL"))
        .stdout(predicate::str::contains("Use Kubernetes"))
        .stdout(predicate::str::contains("supersedes"));

    // 9. Generate TOC
    adrs()
        .current_dir(temp.path())
        .args(["generate", "toc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Record architecture decisions"))
        .stdout(predicate::str::contains("Use PostgreSQL"))
        .stdout(predicate::str::contains("Use Kubernetes"));

    // 10. Generate graph with links
    adrs()
        .current_dir(temp.path())
        .args(["generate", "graph"])
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph"))
        .stdout(predicate::str::contains("Supersedes"));

    // 11. Run doctor
    adrs()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .success();

    temp.close().unwrap();
}

#[test]
fn test_doctor_ng_flag_prints_note() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // --ng is a no-op for doctor, but it should not be silently ignored (issue #306).
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "doctor"])
        .assert()
        .success()
        .stderr(predicate::str::contains("--ng has no effect on 'doctor'"));

    temp.close().unwrap();
}

#[test]
fn test_doctor_without_ng_flag_prints_no_note() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .success()
        .stderr(predicate::str::contains("--ng").not());

    temp.close().unwrap();
}

// Tests for [doctor].ignore / --ignore and warnings-as-errors config (issue #316)

/// Nygard-format ADR content that trips only warning-severity collection rules
/// when used to create a numbering gap (mirrors adrs-core's lint.rs test helper).
fn nygard_adr(number: u32, title: &str) -> String {
    format!(
        "# {number}. {title}\n\nDate: 2024-01-01\n\n## Status\n\nAccepted\n\n## Context\n\nSome context.\n\n## Decision\n\nA decision.\n\n## Consequences\n\nSome consequences.\n"
    )
}

#[test]
fn test_doctor_config_ignore_suppresses_diagnostic() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Create a numbering gap (ADR #1 already exists from init; add #2 and #4,
    // skipping #3) to trip the ADR011 sequential-gap warning.
    fs::write(
        temp.path().join("doc/adr/0002-second.md"),
        nygard_adr(2, "Second"),
    )
    .unwrap();
    fs::write(
        temp.path().join("doc/adr/0004-fourth.md"),
        nygard_adr(4, "Fourth"),
    )
    .unwrap();

    fs::write(
        temp.path().join("adrs.toml"),
        "adr_dir = \"doc/adr\"\n\n[doctor]\nignore = [\"ADR011\"]\n",
    )
    .unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("ADR011").not())
        .stdout(predicate::str::contains("suppressed by ignore rules"));

    temp.close().unwrap();
}

#[test]
fn test_doctor_warnings_as_errors_flag_exits_1_on_warnings_only() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Numbering gap trips only ADR011 (Warning), no errors.
    fs::write(
        temp.path().join("doc/adr/0002-second.md"),
        nygard_adr(2, "Second"),
    )
    .unwrap();
    fs::write(
        temp.path().join("doc/adr/0004-fourth.md"),
        nygard_adr(4, "Fourth"),
    )
    .unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["doctor", "--warnings-as-errors"])
        .assert()
        .failure()
        .code(1);

    temp.close().unwrap();
}

#[test]
fn test_doctor_config_warnings_as_errors_exits_1_without_flag() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        temp.path().join("doc/adr/0002-second.md"),
        nygard_adr(2, "Second"),
    )
    .unwrap();
    fs::write(
        temp.path().join("doc/adr/0004-fourth.md"),
        nygard_adr(4, "Fourth"),
    )
    .unwrap();

    fs::write(
        temp.path().join("adrs.toml"),
        "adr_dir = \"doc/adr\"\n\n[doctor]\nwarnings_as_errors = true\n",
    )
    .unwrap();

    // Plain `adrs doctor`, no --warnings-as-errors flag.
    adrs()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .failure()
        .code(1);

    temp.close().unwrap();
}

#[test]
fn test_doctor_ignore_flag_works_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    // Numbering gap is the only issue in this repo; no adrs.toml is written.
    fs::write(
        temp.path().join("doc/adr/0002-second.md"),
        nygard_adr(2, "Second"),
    )
    .unwrap();
    fs::write(
        temp.path().join("doc/adr/0004-fourth.md"),
        nygard_adr(4, "Fourth"),
    )
    .unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["doctor", "--ignore", "ADR011"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ADR011").not());

    temp.close().unwrap();
}

/// Smoke test for MCP feature availability (default since 0.6.1)
#[test]
#[cfg(feature = "mcp")]
fn test_smoke_mcp_available() {
    // MCP command should be available
    adrs()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP server"))
        .stdout(predicate::str::contains("serve"));
}

/// Smoke test for template system
#[test]
fn test_smoke_template_system() {
    // List templates
    adrs()
        .args(["template", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nygard"))
        .stdout(predicate::str::contains("madr"))
        .stdout(predicate::str::contains("full"))
        .stdout(predicate::str::contains("minimal"))
        .stdout(predicate::str::contains("bare"));

    // Show each format
    adrs()
        .args(["template", "show", "nygard"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Status"));

    adrs()
        .args(["template", "show", "madr"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status:"));

    // Show variants
    adrs()
        .args(["template", "show", "nygard", "--variant", "minimal"])
        .assert()
        .success();

    adrs()
        .args(["template", "show", "madr", "--variant", "bare"])
        .assert()
        .success();
}

/// Smoke test for shell completions
#[test]
fn test_smoke_completions() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        adrs().args(["completions", shell]).assert().success();
    }
}

/// Smoke test for cheatsheet
#[test]
fn test_smoke_cheatsheet() {
    adrs()
        .arg("cheatsheet")
        .assert()
        .success()
        .stdout(predicate::str::contains("GETTING STARTED"))
        .stdout(predicate::str::contains("CREATING ADRs"))
        .stdout(predicate::str::contains("SUPERSEDING"))
        .stdout(predicate::str::contains("STATUS"))
        .stdout(predicate::str::contains("SEARCHING"));
}

/// Smoke test for NextGen mode workflow
#[test]
fn test_smoke_nextgen_workflow() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize in NextGen mode
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "init"])
        .assert()
        .success();

    // Create ADR with tags in NextGen mode
    adrs()
        .current_dir(temp.path())
        .args([
            "--ng",
            "new",
            "--no-edit",
            "-t",
            "database,infrastructure",
            "Use PostgreSQL",
        ])
        .assert()
        .success();

    // Verify YAML frontmatter was created with tags
    let content = fs::read_to_string(temp.path().join("doc/adr/0002-use-postgresql.md")).unwrap();
    assert!(content.starts_with("---"));
    assert!(content.contains("number: 2"));
    assert!(content.contains("title: Use PostgreSQL"));
    assert!(content.contains("status: proposed"));
    assert!(content.contains("tags:"));
    assert!(content.contains("database"));
    assert!(content.contains("infrastructure"));

    // List by tag works
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "list", "--tag", "database"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0002-use-postgresql.md"));

    // Create another ADR without tags - should not appear in tag filter
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "new", "--no-edit", "Use Redis"])
        .assert()
        .success();

    // Tag filter should only show the tagged ADR
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "list", "--tag", "database"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0002-use-postgresql.md"))
        .stdout(predicate::str::contains("0003-use-redis.md").not());

    temp.close().unwrap();
}

/// `new --deciders/--consulted/--informed` writes MADR participant frontmatter in ng mode.
#[test]
fn test_new_madr_participant_flags() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["--ng", "init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args([
            "--ng",
            "new",
            "--no-edit",
            "--deciders",
            "Alice, Bob",
            "--consulted",
            "Security Team",
            "--informed",
            "Engineering",
            "Use OAuth2",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("doc/adr/0002-use-oauth2.md")).unwrap();
    assert!(content.starts_with("---"));
    assert!(content.contains("decision-makers:"));
    assert!(content.contains("Alice"));
    assert!(content.contains("Bob"));
    assert!(content.contains("consulted:"));
    assert!(content.contains("Security Team"));
    assert!(content.contains("informed:"));
    assert!(content.contains("Engineering"));

    // Decider filter finds the ADR.
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "list", "--decider", "Alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0002-use-oauth2.md"));

    temp.close().unwrap();
}

/// The MADR participant flags require ng mode and error clearly in legacy mode.
#[test]
fn test_new_madr_participant_flags_require_ng() {
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "--deciders", "Alice", "Use OAuth2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--ng mode"));

    temp.close().unwrap();
}

// ============================================================================
// -C Relative Path Resolution
// ============================================================================

#[test]
fn test_generate_book_with_cwd_flag_relative_output() {
    // Regression test: `adrs -C <dir> generate book` must write its output
    // relative to the -C directory, not the process cwd, and must not
    // clobber an existing book/ in the process cwd.
    let repo = assert_fs::TempDir::new().unwrap();
    let elsewhere = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    // Pre-existing book/ in the process cwd that must survive untouched
    let sentinel = "# pre-existing book.toml, do not overwrite\n";
    elsewhere.child("book/src").create_dir_all().unwrap();
    elsewhere
        .child("book/book.toml")
        .write_str(sentinel)
        .unwrap();

    adrs()
        .current_dir(elsewhere.path())
        .args(["-C", repo.path().to_str().unwrap(), "generate", "book"])
        .assert()
        .success();

    // Output lands in the -C directory
    repo.child("book/book.toml")
        .assert(predicate::path::exists());
    repo.child("book/src/SUMMARY.md")
        .assert(predicate::path::exists());
    repo.child("book/src/0001-record-architecture-decisions.md")
        .assert(predicate::path::exists());

    // The process cwd's book/ is untouched
    elsewhere.child("book/book.toml").assert(sentinel);
    elsewhere
        .child("book/src/SUMMARY.md")
        .assert(predicate::path::missing());

    repo.close().unwrap();
    elsewhere.close().unwrap();
}

#[test]
fn test_import_json_with_cwd_flag_relative_paths() {
    // Both the JSON file argument and --dir must resolve against the -C
    // directory, not the process cwd.
    let workdir = assert_fs::TempDir::new().unwrap();
    let elsewhere = assert_fs::TempDir::new().unwrap();

    // Build a source repo and export it to JSON inside workdir
    adrs()
        .current_dir(workdir.path())
        .arg("init")
        .assert()
        .success();

    let export_out = adrs()
        .current_dir(workdir.path())
        .args(["export", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    workdir
        .child("adrs.json")
        .write_binary(&export_out)
        .unwrap();

    // Run from elsewhere with -C and relative file + --dir arguments
    adrs()
        .current_dir(elsewhere.path())
        .args([
            "-C",
            workdir.path().to_str().unwrap(),
            "import",
            "json",
            "--dir",
            "imported",
            "adrs.json",
        ])
        .assert()
        .success();

    workdir
        .child("imported/0001-record-architecture-decisions.md")
        .assert(predicate::path::exists());
    elsewhere
        .child("imported")
        .assert(predicate::path::missing());

    workdir.close().unwrap();
    elsewhere.close().unwrap();
}

#[test]
fn test_export_json_with_cwd_flag_relative_dir() {
    // A relative --dir must resolve against the -C directory.
    let workdir = assert_fs::TempDir::new().unwrap();
    let elsewhere = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(workdir.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(elsewhere.path())
        .args([
            "-C",
            workdir.path().to_str().unwrap(),
            "export",
            "json",
            "--dir",
            "doc/adr",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Record architecture decisions"));

    workdir.close().unwrap();
    elsewhere.close().unwrap();
}

#[test]
fn test_generate_toc_with_cwd_flag_relative_intro() {
    // A relative --intro file must resolve against the -C directory.
    let workdir = assert_fs::TempDir::new().unwrap();
    let elsewhere = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(workdir.path())
        .arg("init")
        .assert()
        .success();

    workdir
        .child("intro.md")
        .write_str("Intro from workdir\n")
        .unwrap();

    adrs()
        .current_dir(elsewhere.path())
        .args([
            "-C",
            workdir.path().to_str().unwrap(),
            "generate",
            "toc",
            "--intro",
            "intro.md",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Intro from workdir"));

    workdir.close().unwrap();
    elsewhere.close().unwrap();
}

#[test]
fn test_generate_book_relative_output_without_cwd_flag() {
    // Without -C, a relative --output still resolves against the process
    // cwd and prints the path as given.
    let temp = assert_fs::TempDir::new().unwrap();

    adrs()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["generate", "book", "--output", "mybook"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mybook"));

    temp.child("mybook/book.toml")
        .assert(predicate::path::exists());
    temp.child("mybook/src/SUMMARY.md")
        .assert(predicate::path::exists());

    temp.close().unwrap();
}
