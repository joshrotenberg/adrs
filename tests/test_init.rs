use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use assert_fs::prelude::*;

#[test]
#[serial_test::serial]
fn test_init_default() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .stdout("doc/adr/0001-record-architecture-decisions.md\n")
        .success();

    temp.child("doc/adr/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    assert_eq!(
        std::fs::read_to_string(format!("{}/.adr-dir", temp.path().to_str().unwrap())).unwrap(),
        "doc/adr"
    );
}

#[test]
#[serial_test::serial]
fn test_init_with_directory() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg(temp.path())
        .assert()
        .success();

    temp.child("0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    assert_eq!(
        std::fs::read_to_string(format!("{}/.adr-dir", temp.path().to_str().unwrap())).unwrap(),
        temp.path().to_str().unwrap()
    );
}

#[test]
#[serial_test::serial]
fn test_init_with_file_already_in_directory() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!(
            "{}/0001-record-architecture-decisions.md",
            temp.path().to_str().unwrap()
        ))
        .unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg(temp.path())
        .assert()
        .success();

    temp.child("0002-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    assert_eq!(
        std::fs::read_to_string(format!("{}/.adr-dir", temp.path().to_str().unwrap())).unwrap(),
        temp.path().to_str().unwrap()
    );
}

#[test]
#[serial_test::serial]
fn test_init_issue_4() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg("docs/ADRs")
        .assert()
        .success();

    temp.child("docs/ADRs/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    assert_eq!(
        std::fs::read_to_string(format!("{}/.adr-dir", temp.path().to_str().unwrap())).unwrap(),
        "docs/ADRs"
    );

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("foo")
        .assert()
        .success();
}
