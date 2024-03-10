use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
#[serial_test::serial]
fn test_config() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("config")
        .assert()
        .stdout(predicate::str::contains("adrs_bin_dir=").and(predicate::str::contains("adrs_template_dir=embedded")));
}
