use std::path::Path;

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
#[serial_test::serial]
fn test_list() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    temp.child(".adr-dir").assert(predicates::path::exists());

    let path = Path::new("doc/adr/0001-record-architecture-decisions.md\n");
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(path.to_str().unwrap());

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Another ADR")
        .assert()
        .success();

    let path =
        Path::new("doc/adr/0001-record-architecture-decisions.md\ndoc/adr/0002-another-adr.md\n");
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("list")
        .assert()
        .stdout(path.to_str().unwrap());
}

#[test]
#[serial_test::serial]
fn test_list_alternate_adr_dir() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg("docs/ADRs")
        .assert()
        .success();

    temp.child(".adr-dir").assert(predicates::path::exists());

    let s = &std::fs::read_to_string(Path::new(temp.path()).join(".adr-dir")).unwrap();

    assert_eq!(s, "docs/ADRs");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("list")
        .assert()
        .stdout("docs/ADRs/0001-record-architecture-decisions.md\n");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Another ADR")
        .assert()
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("list")
        .assert()
        .stdout("docs/ADRs/0001-record-architecture-decisions.md\ndocs/ADRs/0002-another-adr.md\n");
}
