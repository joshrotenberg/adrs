use std::path::Path;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use pulldown_cmark::Event;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;

#[test]
#[serial_test::serial]
fn test_link() {
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
        .arg("link")
        .arg("2")
        .arg("Amends")
        .arg("1")
        .arg("Amended by")
        .assert()
        .success();

    let s = &std::fs::read_to_string(
        Path::new(temp.path())
            .join("doc/adr")
            .join("0002-test-new.md"),
    )
    .unwrap();

    let mut in_status = false;
    let predicate_fn = predicate::str::contains("Accepted").or(predicate::str::contains(
        "Amends [1. Record architecture decisions](0001-record-architecture-decisions.md)",
    ));

    let events = Parser::new(s).into_offset_iter();
    for (event, offset) in events {
        match event {
            Event::Start(Tag::Heading(HeadingLevel::H2, _, _)) => {
                in_status = s[offset.clone()].starts_with("## Status");
            }
            _ => {}
        };
        if in_status {
            if let Event::End(Tag::Paragraph) = event {
                assert_eq!(true, predicate_fn.eval(&s[offset]));
            }
        }
    }
}
