use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
#[serial_test::serial]
fn test_link() {
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
        .arg("new")
        .arg("Test new")
        .assert()
        .success();

    temp.child("doc/adr/0002-test-new.md")
        .assert(predicates::path::exists());

    // let s = &std::fs::read_to_string(
    //     Path::new(temp.path())
    //         .join("doc/adr")
    //         .join("0002-test-new.md"),
    // )
    // .unwrap();

    // let events = Parser::new(&s).into_offset_iter();
    // for (event, offset) in events {
    //     if let Event::End(Tag::Heading(HeadingLevel::H1, _, _)) = event {
    //         assert_eq!(&s[offset], "# 2. Test new\n");
    //     }
    // }

    // Command::cargo_bin("adrs")
    //     .unwrap()
    //     .arg("link")
    //     .arg("2")
    //     .arg("1")
    //     .assert()
    //     .success();

    // let s = &std::fs::read_to_string(
    //     Path::new(temp.path())
    //         .join("doc/adr")
    //         .join("0002-test-new.md"),
    // )
    // .unwrap();

    // let events = Parser::new(&s).into_offset_iter();
    // for (event, offset) in events {
    //     if let Event::End(Tag::Heading(HeadingLevel::H1, _, _)) = event {
    //         assert_eq!(&s[offset], "# 2. Test new\n\n## Links\n\n- [1. Record architecture decisions](0001-record-architecture-decisions.md)\n");
    //     }
    // }
}
