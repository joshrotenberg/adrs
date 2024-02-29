use assert_cmd::Command;
use assert_fs::TempDir;

#[test]
#[serial_test::serial]
fn test_list() {
    let tmp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("list")
        .assert()
        .stdout("doc/adr/0001-record-architecture-decisions.md\n");

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
        .stdout("doc/adr/0001-record-architecture-decisions.md\ndoc/adr/0002-another-adr.md\n");
}
