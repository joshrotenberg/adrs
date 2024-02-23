use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use std::path::Path;

#[test]
fn test_init_default() {
    let tmp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("doc/adr")
        .join("0001-record-architecture-decisions.md")
        .exists());
}

#[test]
fn test_init_with_directory() {
    let tmp_dir = TempDir::new().unwrap();
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg(tmp_dir.path())
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("0001-record-architecture-decisions.md")
        .exists());
}

#[test]
fn test_init_with_file_already_in_directory() {
    let tmp_dir = TempDir::new().unwrap();
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!(
            "{}/0001-record-architecture-decisions.md",
            tmp_dir.path().to_str().unwrap()
        ))
        .unwrap();
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .arg(tmp_dir.path())
        .assert()
        .success();

    assert!(Path::new(tmp_dir.path())
        .join("0002-record-architecture-decisions.md")
        .exists());
}
