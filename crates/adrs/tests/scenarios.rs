//! Scenario tests - end-to-end workflows simulating real user behavior.
//!
//! These tests exercise complete user workflows rather than individual commands,
//! catching integration issues that unit tests might miss.

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

fn adrs() -> Command {
    Command::cargo_bin("adrs").unwrap()
}

// ============================================================================
// Scenario: New Project Setup
// ============================================================================

/// User initializes a new project and creates their first few ADRs.
#[test]
fn scenario_new_project_setup() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Step 1: Initialize repository
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    // Verify .adr-dir was created
    let adr_dir_file = temp.child(".adr-dir");
    adr_dir_file.assert(predicate::path::exists());
    adr_dir_file.assert(predicate::str::contains("doc/adr"));

    // Verify initial ADR exists
    let first_adr = temp.child("doc/adr/0001-record-architecture-decisions.md");
    first_adr.assert(predicate::path::exists());

    // Step 2: List shows the initial ADR (outputs file paths)
    adrs()
        .current_dir(temp.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0001-record-architecture-decisions.md",
        ));

    // Step 3: Create a second ADR
    adrs()
        .current_dir(temp.path())
        .args(["new", "Use PostgreSQL for persistence"])
        .env("EDITOR", "true")
        .assert()
        .success();

    let second_adr = temp.child("doc/adr/0002-use-postgresql-for-persistence.md");
    second_adr.assert(predicate::path::exists());

    // Step 4: List shows both ADRs
    adrs()
        .current_dir(temp.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0001-record-architecture-decisions.md",
        ))
        .stdout(predicate::str::contains(
            "0002-use-postgresql-for-persistence.md",
        ));

    // Step 5: Config shows correct state
    adrs()
        .current_dir(temp.path())
        .args(["config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("doc/adr"));

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Superseding a Decision
// ============================================================================

/// User creates an ADR, then later supersedes it with a new decision.
#[test]
fn scenario_supersede_decision() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize and create initial ADR
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Use MySQL for persistence"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Verify ADR 2 exists with Proposed status
    let mysql_adr = temp.child("doc/adr/0002-use-mysql-for-persistence.md");
    mysql_adr.assert(predicate::path::exists());
    let mysql_content = fs::read_to_string(mysql_adr.path()).unwrap();
    assert!(mysql_content.contains("Proposed"));

    // Step 1: Create superseding ADR
    adrs()
        .current_dir(temp.path())
        .args([
            "new",
            "--supersedes",
            "2",
            "Use PostgreSQL instead of MySQL",
        ])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 2: Verify new ADR exists and has supersedes link
    let postgres_adr = temp.child("doc/adr/0003-use-postgresql-instead-of-mysql.md");
    postgres_adr.assert(predicate::path::exists());
    let postgres_content = fs::read_to_string(postgres_adr.path()).unwrap();
    assert!(
        postgres_content.contains("Supersedes"),
        "New ADR should contain 'Supersedes' link"
    );

    // Step 3: Verify old ADR is now superseded
    let mysql_content_after = fs::read_to_string(mysql_adr.path()).unwrap();
    assert!(
        mysql_content_after.contains("Superseded"),
        "Old ADR should have 'Superseded' status"
    );
    assert!(
        mysql_content_after.contains("Superseded by"),
        "Old ADR should have 'Superseded by' link"
    );

    // Step 4: List shows all ADRs (file paths)
    adrs()
        .current_dir(temp.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0002-use-mysql-for-persistence.md",
        ))
        .stdout(predicate::str::contains(
            "0003-use-postgresql-instead-of-mysql.md",
        ));

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Linking Related Decisions
// ============================================================================

/// User creates multiple ADRs and links them together.
#[test]
fn scenario_link_decisions() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize and create two ADRs
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Use REST API"])
        .env("EDITOR", "true")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Use JSON for API responses"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 1: Link ADR 3 to ADR 2
    adrs()
        .current_dir(temp.path())
        .args(["link", "3", "Extends", "2", "Extended by"])
        .assert()
        .success();

    // Step 2: Verify ADR 3 has the forward link
    let json_adr = temp.child("doc/adr/0003-use-json-for-api-responses.md");
    let json_content = fs::read_to_string(json_adr.path()).unwrap();
    assert!(
        json_content.contains("Extends"),
        "ADR 3 should contain 'Extends' link"
    );

    // Step 3: Verify ADR 2 has the reverse link
    let rest_adr = temp.child("doc/adr/0002-use-rest-api.md");
    let rest_content = fs::read_to_string(rest_adr.path()).unwrap();
    assert!(
        rest_content.contains("Extended by"),
        "ADR 2 should contain 'Extended by' link"
    );

    temp.close().unwrap();
}

/// User links two ADRs using simplified syntax (reverse link auto-derived).
#[test]
fn scenario_link_simplified_syntax() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize and create two ADRs
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Original decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Updated decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 1: Link ADR 3 supersedes ADR 2 (without specifying reverse link)
    adrs()
        .current_dir(temp.path())
        .args(["link", "3", "Supersedes", "2"])
        .assert()
        .success();

    // Step 2: Verify ADR 3 has the forward link
    let adr3 = temp.child("doc/adr/0003-updated-decision.md");
    let adr3_content = fs::read_to_string(adr3.path()).unwrap();
    assert!(
        adr3_content.contains("Supersedes"),
        "ADR 3 should contain 'Supersedes' link"
    );

    // Step 3: Verify ADR 2 has the auto-derived reverse link
    let adr2 = temp.child("doc/adr/0002-original-decision.md");
    let adr2_content = fs::read_to_string(adr2.path()).unwrap();
    assert!(
        adr2_content.contains("Superseded by"),
        "ADR 2 should contain auto-derived 'Superseded by' link"
    );

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Working from Subdirectory
// ============================================================================

/// User runs commands from a subdirectory of their project.
#[test]
fn scenario_subdirectory_workflow() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize in root
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    // Create a subdirectory structure
    let src_dir = temp.child("src/components");
    src_dir.create_dir_all().unwrap();

    // Step 1: Create ADR from subdirectory
    adrs()
        .current_dir(src_dir.path())
        .args(["new", "Component architecture"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Verify ADR was created in correct location (project root's doc/adr)
    let adr = temp.child("doc/adr/0002-component-architecture.md");
    adr.assert(predicate::path::exists());

    // Step 2: List from subdirectory
    adrs()
        .current_dir(src_dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0002-component-architecture.md"));

    // Step 3: Config from subdirectory shows project root
    adrs()
        .current_dir(src_dir.path())
        .args(["config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("doc/adr"));

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Generate Documentation
// ============================================================================

/// User generates table of contents and graph after creating several ADRs.
#[test]
fn scenario_generate_documentation() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize and create several ADRs with links
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Use microservices architecture"])
        .env("EDITOR", "true")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Use Kubernetes for orchestration"])
        .env("EDITOR", "true")
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["link", "3", "Extends", "2", "Extended by"])
        .assert()
        .success();

    // Step 1: Generate table of contents
    let toc_output = adrs()
        .current_dir(temp.path())
        .args(["generate", "toc"])
        .assert()
        .success();

    let toc = String::from_utf8_lossy(&toc_output.get_output().stdout);
    assert!(toc.contains("Record architecture decisions"));
    assert!(toc.contains("Use microservices architecture"));
    assert!(toc.contains("Use Kubernetes for orchestration"));

    // Step 2: Generate graph
    let graph_output = adrs()
        .current_dir(temp.path())
        .args(["generate", "graph"])
        .assert()
        .success();

    let graph = String::from_utf8_lossy(&graph_output.get_output().stdout);
    assert!(graph.contains("digraph"));
    assert!(graph.contains("Extends"));

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Custom Directory
// ============================================================================

/// User initializes with a custom ADR directory.
#[test]
fn scenario_custom_directory() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Step 1: Initialize with custom directory
    adrs()
        .current_dir(temp.path())
        .args(["init", "decisions/arch"])
        .assert()
        .success();

    // Verify .adr-dir points to custom directory
    let adr_dir_file = temp.child(".adr-dir");
    adr_dir_file.assert(predicate::str::contains("decisions/arch"));

    // Verify ADR was created in custom directory
    let first_adr = temp.child("decisions/arch/0001-record-architecture-decisions.md");
    first_adr.assert(predicate::path::exists());

    // Step 2: Create new ADR - should go to custom directory
    adrs()
        .current_dir(temp.path())
        .args(["new", "Custom location test"])
        .env("EDITOR", "true")
        .assert()
        .success();

    let second_adr = temp.child("decisions/arch/0002-custom-location-test.md");
    second_adr.assert(predicate::path::exists());

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Doctor Finds Issues
// ============================================================================

/// User runs doctor to find issues in their ADR repository.
#[test]
fn scenario_doctor_finds_issues() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize repository
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "First decision"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 1: Doctor should pass on healthy repo
    adrs()
        .current_dir(temp.path())
        .args(["doctor"])
        .assert()
        .success();

    // Step 2: Create a gap in numbering by manually creating ADR 5
    let adr_dir = temp.child("doc/adr");
    fs::write(
        adr_dir.path().join("0005-gap-in-numbering.md"),
        "# 5. Gap in numbering\n\nDate: 2024-01-15\n\n## Status\n\nProposed\n\n## Context\n\nTest\n\n## Decision\n\nTest\n\n## Consequences\n\nTest\n",
    )
    .unwrap();

    // Step 3: Doctor should find the gap (reports as "Missing ADR number X")
    adrs()
        .current_dir(temp.path())
        .args(["doctor"])
        .assert()
        .stdout(predicate::str::contains("Missing ADR number"));

    temp.close().unwrap();
}

// ============================================================================
// Scenario: MADR Format Workflow
// ============================================================================

/// User creates ADRs using MADR format.
#[test]
fn scenario_madr_format() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize repository
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    // Step 1: Create ADR with MADR format
    adrs()
        .current_dir(temp.path())
        .args(["new", "--format", "madr", "Use MADR format for decisions"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 2: Verify MADR structure
    let madr_adr = temp.child("doc/adr/0002-use-madr-format-for-decisions.md");
    madr_adr.assert(predicate::path::exists());
    let content = fs::read_to_string(madr_adr.path()).unwrap();

    // MADR has different section structure
    assert!(
        content.contains("Context and Problem Statement")
            || content.contains("Decision Outcome")
            || content.contains("Considered Options"),
        "MADR format should have MADR-specific sections"
    );

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Config-Driven Template Selection
// ============================================================================

/// User configures template format and variant in adrs.toml and expects
/// `adrs new` to use those settings without CLI flags.
#[test]
fn scenario_config_driven_template() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Step 1: Initialize with nextgen mode
    adrs()
        .current_dir(temp.path())
        .args(["--ng", "init"])
        .assert()
        .success();

    // Step 2: Write adrs.toml with format=madr, variant=minimal, mode=nextgen
    fs::write(
        temp.path().join("adrs.toml"),
        r#"
adr_dir = "doc/adr"
mode = "nextgen"

[templates]
format = "madr"
variant = "minimal"
"#,
    )
    .unwrap();

    // Step 3: Verify config shows the template settings
    adrs()
        .current_dir(temp.path())
        .args(["config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Template format: madr"))
        .stdout(predicate::str::contains("Template variant: minimal"));

    // Step 4: Create ADR without any --format or --variant flags
    adrs()
        .current_dir(temp.path())
        .args(["new", "--no-edit", "Config driven template test"])
        .assert()
        .success();

    // Step 5: Verify the ADR uses MADR minimal template
    let adr = temp.child("doc/adr/0002-config-driven-template-test.md");
    adr.assert(predicate::path::exists());
    let content = fs::read_to_string(adr.path()).unwrap();

    // MADR minimal has these sections
    assert!(
        content.contains("Context and Problem Statement"),
        "Should use MADR format from config"
    );
    assert!(
        content.contains("Considered Options"),
        "Should use MADR format from config"
    );
    assert!(
        content.contains("Decision Outcome"),
        "Should use MADR format from config"
    );
    // MADR minimal does NOT have these sections (that's what makes it "minimal")
    assert!(
        !content.contains("Decision Drivers"),
        "Minimal variant should not include Decision Drivers"
    );
    assert!(
        !content.contains("Pros and Cons"),
        "Minimal variant should not include Pros and Cons"
    );

    // Step 6: CLI flags should still override config
    adrs()
        .current_dir(temp.path())
        .args([
            "new",
            "--no-edit",
            "--format",
            "nygard",
            "CLI override test",
        ])
        .assert()
        .success();

    let override_adr = temp.child("doc/adr/0003-cli-override-test.md");
    override_adr.assert(predicate::path::exists());
    let override_content = fs::read_to_string(override_adr.path()).unwrap();

    // Nygard format has different sections than MADR
    assert!(
        override_content.contains("## Context"),
        "CLI --format flag should override config"
    );
    assert!(
        override_content.contains("## Decision"),
        "CLI --format flag should override config"
    );
    assert!(
        override_content.contains("## Consequences"),
        "CLI --format flag should override config"
    );

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Edit Existing ADR
// ============================================================================

/// User edits an existing ADR by number and by fuzzy title match.
#[test]
fn scenario_edit_workflow() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize and create ADRs
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    adrs()
        .current_dir(temp.path())
        .args(["new", "Database selection"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 1: Edit by number (using 'true' as no-op editor)
    adrs()
        .current_dir(temp.path())
        .args(["edit", "2"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 2: Edit by fuzzy title match
    adrs()
        .current_dir(temp.path())
        .args(["edit", "database"])
        .env("EDITOR", "true")
        .assert()
        .success();

    temp.close().unwrap();
}

// ============================================================================
// Scenario: Status Change Workflow
// ============================================================================

/// User creates ADRs and changes their status through the decision lifecycle.
#[test]
fn scenario_status_workflow() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Setup: Initialize repository
    adrs()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    // Step 1: Create a new ADR (starts as Proposed)
    adrs()
        .current_dir(temp.path())
        .args(["new", "Use Redis for caching"])
        .env("EDITOR", "true")
        .assert()
        .success();

    let redis_adr = temp.child("doc/adr/0002-use-redis-for-caching.md");
    redis_adr.assert(predicate::path::exists());
    let content = fs::read_to_string(redis_adr.path()).unwrap();
    assert!(
        content.contains("Proposed"),
        "New ADR should start with Proposed status"
    );

    // Step 2: After team review, mark as accepted
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "accepted"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to Accepted"));

    let content = fs::read_to_string(redis_adr.path()).unwrap();
    assert!(
        content.contains("Accepted"),
        "ADR status should be updated to Accepted"
    );

    // Step 3: Create a better alternative
    adrs()
        .current_dir(temp.path())
        .args(["new", "Use Memcached for caching"])
        .env("EDITOR", "true")
        .assert()
        .success();

    // Step 4: Mark old decision as superseded with automatic link
    adrs()
        .current_dir(temp.path())
        .args(["status", "2", "superseded", "--by", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("superseded by 3"));

    // Verify old ADR has superseded status and link
    let content = fs::read_to_string(redis_adr.path()).unwrap();
    assert!(
        content.contains("Superseded"),
        "Old ADR should be marked as Superseded"
    );
    assert!(
        content.contains("Superseded by [3."),
        "Old ADR should have superseded-by link"
    );

    // Step 5: Mark new decision as accepted
    adrs()
        .current_dir(temp.path())
        .args(["status", "3", "accepted"])
        .assert()
        .success();

    let memcached_adr = temp.child("doc/adr/0003-use-memcached-for-caching.md");
    let content = fs::read_to_string(memcached_adr.path()).unwrap();
    assert!(content.contains("Accepted"), "New ADR should be Accepted");

    // Step 6: Later decide to deprecate caching altogether
    adrs()
        .current_dir(temp.path())
        .args(["status", "3", "deprecated"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status changed to Deprecated"));

    let content = fs::read_to_string(memcached_adr.path()).unwrap();
    assert!(
        content.contains("Deprecated"),
        "ADR should be marked as Deprecated"
    );

    temp.close().unwrap();
}
