use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};

#[test]
#[serial_test::serial]
fn test_new_default() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    temp.child("doc/adr/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test new")
        .assert()
        .success();

    temp.child("doc/adr/0002-test-new.md")
        .assert(predicates::path::exists());

    let s = &std::fs::read_to_string(
        Path::new(temp.path())
            .join("doc/adr")
            .join("0002-test-new.md"),
    )
    .unwrap();

    let events = Parser::new(s).into_offset_iter();
    for (event, offset) in events {
        if let Event::End(Tag::Heading(HeadingLevel::H1, _, _)) = event {
            assert_eq!(&s[offset], "# 2. Test new\n");
        }
    }
}

#[test]
#[serial_test::serial]
fn test_new_superseded() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    temp.child("doc/adr/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test new")
        .assert()
        .success();

    temp.child("doc/adr/0002-test-new.md")
        .assert(predicates::path::exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        // `superceded` is a hidden alias for the correct spelling: `superseded`
        // and is maintained for backwards compatibility
        .arg("--superceded")
        .arg("2")
        .arg("Test new")
        .assert()
        .success();

    let s = &std::fs::read_to_string(
        Path::new(temp.path())
            .join("doc/adr")
            .join("0003-test-new.md"),
    )
    .unwrap();

    let events = Parser::new(s).into_offset_iter();
    for (event, offset) in events {
        if let Event::End(Tag::Heading(HeadingLevel::H1, _, _)) = event {
            assert_eq!(&s[offset], "# 3. Test new\n");
        }
    }
}

#[test]
#[serial_test::serial]
fn test_new_link() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    temp.child("doc/adr/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("--link")
        .arg("1:Amends:Amended by")
        .arg("Test Link")
        .assert()
        .success();

    temp.child("doc/adr/0002-test-link.md")
        .assert(predicates::path::exists());

    let s = &std::fs::read_to_string(
        Path::new(temp.path())
            .join("doc/adr")
            .join("0002-test-link.md"),
    )
    .unwrap();

    let mut in_status = false;
    let predicate_fn = predicate::str::contains("Accepted").or(predicate::str::contains(
        "Amends [1. Record architecture decisions](0001-record-architecture-decisions.md)",
    ));

    let events = Parser::new(s).into_offset_iter();
    for (event, offset) in events {
        if let Event::Start(Tag::Heading(HeadingLevel::H2, _, _)) = event {
            in_status = s[offset.clone()].starts_with("## Status");
        }
        if in_status {
            if let Event::End(Tag::Paragraph) = event {
                assert!(predicate_fn.eval(&s[offset]));
            }
        }
    }
}

#[test]
#[serial_test::serial]
fn test_new_no_current_dir() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test new without init")
        .assert()
        .success();

    temp.child("doc/adr/0001-test-new-without-init.md")
        .assert(predicates::path::exists());
}

#[test]
#[serial_test::serial]
fn test_new_template() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    let default_location_template_path =
        Path::new(temp.path()).join("doc/adr/templates/template.md");
    std::fs::create_dir_all(default_location_template_path.parent().unwrap()).unwrap();
    let mut default_location_template = File::create(default_location_template_path).unwrap();
    default_location_template
        .write_all(b"template default location")
        .unwrap();

    let custom_location_template_path = Path::new(temp.path()).join("doc/adr/templates/custom.md");
    let mut custom_location_template = File::create(&custom_location_template_path).unwrap();
    custom_location_template
        .write_all(b"template custom location")
        .unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test template default location")
        .assert()
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("-T")
        .arg(custom_location_template_path)
        .arg("Test template custom location")
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(
            Path::new(temp.path()).join("doc/adr/0002-test-template-default-location.md"),
        )
        .unwrap(),
        "template default location"
    );

    assert_eq!(
        std::fs::read_to_string(
            Path::new(temp.path()).join("doc/adr/0003-test-template-custom-location.md"),
        )
        .unwrap(),
        "template custom location"
    );
}
