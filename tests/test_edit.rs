use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
#[serial_test::serial]
fn test_edit() {
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
        .arg("edit")
        .arg("record")
        .assert()
        .success();
}
