use assert_cmd::Command;
use assert_fs::{
    fixture::{FileWriteStr, PathChild},
    TempDir,
};

#[test]
#[serial_test::serial]
fn test_generate_toc() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
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
        .arg("generate")
        .arg("toc")
        .assert()
        .stdout("# Architecture Decision Records\n\n* [1. Record architecture decisions](0001-record-architecture-decisions.md)\n* [2. Test new](0002-test-new.md)\n")
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("toc")
        .arg("--ordered")
        .assert()
        .stdout("# Architecture Decision Records\n\n1. [Record architecture decisions](0001-record-architecture-decisions.md)\n1. [Test new](0002-test-new.md)\n")
        .success();

    temp.child("intro.txt").write_str("intro text").unwrap();
    temp.child("outro.txt").write_str("outro text").unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("toc")
        .arg("--intro")
        .arg("intro.txt")
        .arg("--outro")
        .arg("outro.txt")
        .arg("--prefix")
        .arg("prefix")
        .assert().stdout("# Architecture Decision Records\n\nintro text\n* [1. Record architecture decisions](prefix/0001-record-architecture-decisions.md)\n* [2. Test new](prefix/0002-test-new.md)\n\noutro text\n")
        .success();
}

#[test]
#[serial_test::serial]
fn test_generate_graph() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
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

    let graph = "digraph {\n  node [shape=plaintext]\n  subgraph {\n\t_1 [label=\"1. Record architecture decisions\"; URL=\"0001-record-architecture-decisions.html\"];\n\t_2 [label=\"2. Test new\"; URL=\"0002-test-new.html\"];\n\t_1 -> _2 [style=\"dotted\", weight=1];\n  }\n}\n";
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("graph")
        .assert()
        .success()
        .stdout(graph);

    let graph = "digraph {\n  node [shape=plaintext]\n  subgraph {\n\t_1 [label=\"1. Record architecture decisions\"; URL=\"prefix/0001-record-architecture-decisions.pdf\"];\n\t_2 [label=\"2. Test new\"; URL=\"prefix/0002-test-new.pdf\"];\n\t_1 -> _2 [style=\"dotted\", weight=1];\n  }\n}\n";
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("graph")
        .arg("--prefix")
        .arg("prefix")
        .arg("--extension")
        .arg("pdf")
        .assert()
        .success()
        .stdout(graph);
}

#[test]
#[serial_test::serial]
fn test_generate_book() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
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
        .arg("new")
        .arg("Test another")
        .assert()
        .success();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("book")
        .assert()
        .success();

    assert!(temp.child("book").join("book.toml").exists());
    assert!(temp.child("book").join("src").join("SUMMARY.md").exists());
    assert!(temp
        .child("book")
        .join("src")
        .join("0001-record-architecture-decisions.md")
        .exists());
    assert!(temp
        .child("book")
        .join("src")
        .join("0002-test-new.md")
        .exists());
    assert!(temp
        .child("book")
        .join("src")
        .join("0003-test-another.md")
        .exists());
}
