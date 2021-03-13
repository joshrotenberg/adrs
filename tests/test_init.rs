use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use std::path::Path;

#[test]
fn test_init() {
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
