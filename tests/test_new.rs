use std::path::Path;

use assert_cmd::Command;
use assert_fs::TempDir;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};

#[test]
#[serial_test::serial]
fn test_new_default() {
    let tmp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("doc/adr")
        .join("0001-record-architecture-decisions.md")
        .exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test new")
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("doc/adr")
        .join("0002-test-new.md")
        .exists());

    let s = &std::fs::read_to_string(
        Path::new(tmp_dir.path())
            .join("doc/adr")
            .join("0002-test-new.md"),
    )
    .unwrap();

    let events = Parser::new(&s).into_offset_iter();
    for (event, offset) in events {
        match event {
            Event::End(Tag::Heading(HeadingLevel::H1, _, _)) => {
                assert_eq!(&s[offset], "# 2. Test new\n");
            }
            // test more events here
            _ => {}
        }
    }
}

#[test]
#[serial_test::serial]
fn test_new_superceded() {
    let tmp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("doc/adr")
        .join("0001-record-architecture-decisions.md")
        .exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test new")
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("doc/adr")
        .join("0002-test-new.md")
        .exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("--superceded")
        .arg("2")
        .arg("Test new")
        .assert()
        .success();

    let s = &std::fs::read_to_string(
        Path::new(tmp_dir.path())
            .join("doc/adr")
            .join("0003-test-new.md"),
    )
    .unwrap();

    let events = Parser::new(&s).into_offset_iter();
    for (event, offset) in events {
        match event {
            Event::End(Tag::Heading(HeadingLevel::H1, _, _)) => {
                assert_eq!(&s[offset], "# 3. Test new\n");
            }
            Event::End(Tag::Heading(HeadingLevel::H2, _, _)) => {
                dbg!(&s[offset]);
                // assert_eq!(
                //     &s[offset],
                //     "\n\nThis ADR supercedes [ADR 2](0002-test-new.md).\n"
                // );
            }
            // test more events here
            _ => {}
        }
    }
}
