// see https://github.com/npryce/adr-tools/tree/master/tests

use std::fs::read_to_string;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

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
fn test_avoid_octal_numbers() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    let adr_titles = vec![
        "First decision",
        "Second decision",
        "Third decision",
        "Fourth decision",
        "Fifth decision",
        "Sixth decision",
        "Seventh decision",
        "Eighth decision",
        "Ninth decision",
    ];

    adr_titles
        .into_iter()
        .enumerate()
        .for_each(|(index, title)| {
            let file = format!(
                "{:04}-{}.md",
                index + 1,
                title.replace(' ', "-").to_ascii_lowercase()
            );
            Command::cargo_bin("adrs")
                .unwrap()
                .arg("new")
                .arg(title)
                .assert()
                .success()
                .stdout(predicates::str::contains(&file));
            temp.child(format!("doc/adr/{}", file))
                .assert(predicates::path::exists());
        });

    let lines = read_to_string("doc/adr/0009-ninth-decision.md")
        .unwrap()
        .lines()
        .map(String::from)
        .take(7)
        .collect::<Vec<String>>()
        .join("\n");

    let predicate_fn = predicates::str::is_match(
        r"# 9. Ninth decision\n\nDate: \d{4}-\d{2}-\d{2}\n\n## Status\n\nAccepted",
    )
    .unwrap();

    assert!(predicate_fn.eval(&lines));
}

#[test]
#[serial_test::serial]
fn test_create_first_record() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("The First Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0001-the-first-decision.md",
        ));

    temp.child("doc/adr/0001-the-first-decision.md")
        .assert(predicates::path::exists());

    // XXX: test contents
    // let adr = read_to_string("doc/adr/0001-the-first-decision.md").unwrap();
}

#[test]
#[serial_test::serial]
fn test_funny_characters() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Something About Node.JS")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0001-something-about-node-js.md",
        ));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Slash/Slash/Slash/")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0002-slash-slash-slash.md",
        ));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("--")
        .arg("-Bar-")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0003-bar.md"));

    temp.child("doc/adr/0001-something-about-node-js.md")
        .assert(predicates::path::exists());

    temp.child("doc/adr/0002-slash-slash-slash.md")
        .assert(predicates::path::exists());

    temp.child("doc/adr/0003-bar.md")
        .assert(predicates::path::exists());
}

#[test]
#[serial_test::serial]
fn test_generate_contents_with_header_and_footer() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("First Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0001-first-decision.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Second Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0002-second-decision.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Third Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0003-third-decision.md"));

    temp.child("intro.md")
        .write_str("An intro.\n\nMultiple paragraphs.\n")
        .unwrap();

    temp.child("outro.md").write_str("An outro.\n").unwrap();

    let markdown = r#"# Architecture Decision Records

An intro.

Multiple paragraphs.

* [1. First Decision](0001-first-decision.md)
* [2. Second Decision](0002-second-decision.md)
* [3. Third Decision](0003-third-decision.md)

An outro.

"#;
    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("toc")
        .arg("-i")
        .arg("intro.md")
        .arg("-o")
        .arg("outro.md")
        .assert()
        .success()
        .stdout(markdown);
}

#[test]
#[serial_test::serial]
fn test_generate_contents_with_prefix() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("First Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0001-first-decision.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Second Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0002-second-decision.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Third Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0003-third-decision.md"));

    let markdown = r#"# Architecture Decision Records

* [1. First Decision](foo/doc/adr/0001-first-decision.md)
* [2. Second Decision](foo/doc/adr/0002-second-decision.md)
* [3. Third Decision](foo/doc/adr/0003-third-decision.md)
"#;

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("toc")
        .arg("-p")
        .arg("foo/doc/adr")
        .assert()
        .success()
        .stdout(markdown);
}

#[test]
#[serial_test::serial]
fn test_generate_contents() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    std::env::set_var("EDITOR", "cat");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("First Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0001-first-decision.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Second Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0002-second-decision.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("Third Decision")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0003-third-decision.md"));

    let markdown = r#"# Architecture Decision Records

* [1. First Decision](0001-first-decision.md)
* [2. Second Decision](0002-second-decision.md)
* [3. Third Decision](0003-third-decision.md)
"#;

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("toc")
        .assert()
        .success()
        .stdout(markdown);
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
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0001-record-architecture-decisions.md",
        ));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("An idea that seems good at the time")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "doc/adr/0002-an-idea-that-seems-good-at-the-time.md",
        ));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("-s")
        .arg("2")
        .arg("A better idea")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0003-a-better-idea.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("This will work")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0004-this-will-work.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("new")
        .arg("-s")
        .arg("3")
        .arg("The end")
        .assert()
        .success()
        .stdout(predicates::str::contains("doc/adr/0005-the-end.md"));

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("graph")
        .assert()
        .success()
        .stdout("digraph {\n  node [shape=plaintext]\n  subgraph {\n\t_1 [label=\"1. Record architecture decisions\"; URL=\"0001-record-architecture-decisions.html\"];\n\t_2 [label=\"2. An idea that seems good at the time\"; URL=\"0002-an-idea-that-seems-good-at-the-time.html\"];\n\t_1 -> _2 [style=\"dotted\", weight=1];\n\t_3 [label=\"3. A better idea\"; URL=\"0003-a-better-idea.html\"];\n\t_2 -> _3 [style=\"dotted\", weight=1];\n\t_4 [label=\"4. This will work\"; URL=\"0004-this-will-work.html\"];\n\t_3 -> _4 [style=\"dotted\", weight=1];\n\t_5 [label=\"5. The end\"; URL=\"0005-the-end.html\"];\n\t_4 -> _5 [style=\"dotted\", weight=1];\n  }\n  _3 -> _2 [label=\"Supersedes\", weight=0];\n  _5 -> _3 [label=\"Supersedes\", weight=0];\n}\n");

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("generate")
        .arg("graph")
        .arg("-p")
        .arg("http://example.com/")
        .arg("-e")
        .arg(".xxx")
        .assert()
        .success()
    .stdout("digraph {\n  node [shape=plaintext]\n  subgraph {\n\t_1 [label=\"1. Record architecture decisions\"; URL=\"http://example.com/0001-record-architecture-decisions.xxx\"];\n\t_2 [label=\"2. An idea that seems good at the time\"; URL=\"http://example.com/0002-an-idea-that-seems-good-at-the-time.xxx\"];\n\t_1 -> _2 [style=\"dotted\", weight=1];\n\t_3 [label=\"3. A better idea\"; URL=\"http://example.com/0003-a-better-idea.xxx\"];\n\t_2 -> _3 [style=\"dotted\", weight=1];\n\t_4 [label=\"4. This will work\"; URL=\"http://example.com/0004-this-will-work.xxx\"];\n\t_3 -> _4 [style=\"dotted\", weight=1];\n\t_5 [label=\"5. The end\"; URL=\"http://example.com/0005-the-end.xxx\"];\n\t_4 -> _5 [style=\"dotted\", weight=1];\n  }\n  _3 -> _2 [label=\"Supersedes\", weight=0];\n  _5 -> _3 [label=\"Supersedes\", weight=0];\n}\n");
}

#[test]
#[serial_test::serial]
fn test_init_adr_repository() {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .arg("init")
        .assert()
        .success()
        .stdout("doc/adr/0001-record-architecture-decisions.md\n");

    temp.child("doc/adr/0001-record-architecture-decisions.md")
        .assert(predicates::path::exists());

    // TODO: test contents
}
