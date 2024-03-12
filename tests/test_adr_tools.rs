// see https://github.com/npryce/adr-tools/tree/master/tests

use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

use assert_fs::TempDir;

#[test]
#[serial_test::serial]
fn test_alternative_adr_directory() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg("alternative-dir")
        .assert()
        .success()
        .stdout("alternative-dir/0001-record-architecture-decisions.md\n");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Example ADR")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "alternative-dir/0002-example-adr.md",
        ));

    temp.child("alternative-dir")
        .assert(predicates::path::exists());
    temp.child("alternative-dir/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());
    temp.child("alternative-dir/0002-example-adr.md")
        .assert(predicates::path::exists());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("list")
        .assert()
        .success()
        .stdout(
        "alternative-dir/0001-record-architecture-decisions.md\nalternative-dir/0002-example-adr.md\n");
}

#[test]
#[serial_test::serial]
fn test_autocomplete() {
    // TODO
}

#[test]
#[serial_test::serial]
fn test_avoid_octal_numbers() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    let adr_titles = vec![
        "First decision",
        "Second decision",
        "Third decision",
        "Fourth decision",
        "Fifth decision",
        "Sixth decision",
        "Seventh decision",
        "Eighth decision",
        "Ninth decision",
    ];

    adr_titles
        .into_iter()
        .enumerate()
        .for_each(|(index, title)| {
            let file = format!(
                "{:04}-{}.md",
                index + 1,
                title.replace(' ', "-").to_ascii_lowercase()
            );
            Command::cargo_bin("adrs")
                .unwrap()
                .arg("new")
                .arg(title)
                .assert()
                .success()
                .stdout(predicates::str::contains(&file));
            temp.child(format!("doc/adr/{}", file))
                .assert(predicates::path::exists());
        });

    let lines = read_to_string("doc/adr/0009-ninth-decision.md")
        .unwrap()
        .lines()
        .map(String::from)
        .take(7)
        .collect::<Vec<String>>()
        .join("\n");

    let predicate_fn =
        predicates::str::is_match("# 9. Ninth decision\n\nDate: .*\n\n## Status\n\nAccepted")
            .unwrap();

    assert!(predicate_fn.eval(&lines));
}

#[test]
#[serial_test::serial]
fn test_create_first_record() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("The First Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0001-the-first-decision.md",
        ));

    temp.child("doc/adr/0001-the-first-decision.md")
        .assert(predicates::path::exists());

    // XXX: test contents
    // let adr = read_to_string("doc/adr/0001-the-first-decision.md").unwrap();
}

#[test]
#[serial_test::serial]
fn test_funny_characters() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Something About Node.JS")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0001-something-about-node-js.md",
        ));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Slash/Slash/Slash/")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0002-slash-slash-slash.md",
        ));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("--")
        .arg("-Bar-")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0003-bar.md"));

    temp.child("doc/adr/0001-something-about-node-js.md")
        .assert(predicates::path::exists());

    temp.child("doc/adr/0002-slash-slash-slash.md")
        .assert(predicates::path::exists());

    temp.child("doc/adr/0003-bar.md")
        .assert(predicates::path::exists());
}
