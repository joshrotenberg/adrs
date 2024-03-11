use std::path::Path;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
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
fn test_new_superceded() {
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

    let events = Parser::new(s).into_offset_iter();
    for (event, offset) in events {
        if let Event::End(Tag::Heading(HeadingLevel::H1, _, _)) = event {
            assert_eq!(&s[offset], "# 2. Test Link\n");
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
