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
        .stdout(
            predicate::str::contains("adrs_bin_dir=")
                .and(predicate::str::contains("adrs_template_dir=embedded")),
        );
}

#[test]
#[serial_test::serial]
fn test_config_with_embedded_template() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    // std::env::set_var(
    //     "ADRS_TEMPLATE",
    //     temp.path().join("templates").to_str().unwrap(),
    // );
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Test new")
        .assert()
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("config")
        .assert()
        .stdout(
            predicate::str::contains("adrs_template_dir=embedded")
                .and(predicate::str::contains("adrs_dir=doc/adr")),
        );
}
