// see https://github.com/npryce/adr-tools/tree/master/tests

use assert_cmd::Command;
use assert_fs::prelude::*;
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
fn test_avoid_octal_numbers() {}
