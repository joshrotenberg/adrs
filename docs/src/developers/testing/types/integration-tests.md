# Integration Tests

Integration tests verify that modules work together correctly.

## Location

Integration tests are in `tests/` directories:

```
crates/adrs-core/tests/
├── fixtures/           # Test data
├── repository_tests.rs # Repository tests
├── parse_tests.rs      # Parser tests
└── export_tests.rs     # Export tests
```

## Example: Repository Tests

```rust
// tests/repository_tests.rs
use adrs_core::{Repository, Adr, AdrStatus};
use tempfile::tempdir;

#[test]
fn test_create_and_list_adr() {
    let dir = tempdir().unwrap();

    // Initialize
    let repo = Repository::init(dir.path(), None, false).unwrap();

    // Create ADR
    let mut adr = Adr::new(2, "Test Decision");
    adr.context = "Test context".into();
    repo.create(&adr).unwrap();

    // List
    let adrs = repo.list().unwrap();
    assert_eq!(adrs.len(), 2); // Initial + new
    assert_eq!(adrs[1].title, "Test Decision");
}
```

## Example: CLI Tests

```rust
// tests/cli_tests.rs
use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_init_creates_repository() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("adrs")
        .unwrap()
        .args(["init"])
        .current_dir(&dir)
        .assert()
        .success();

    assert!(dir.path().join(".adr-dir").exists());
    assert!(dir.path().join("doc/adr").exists());
}

#[test]
fn test_new_creates_adr() {
    let dir = tempdir().unwrap();

    // Setup
    Command::cargo_bin("adrs")
        .unwrap()
        .args(["init"])
        .current_dir(&dir)
        .assert()
        .success();

    // Create ADR
    Command::cargo_bin("adrs")
        .unwrap()
        .args(["new", "--no-edit", "Test Decision"])
        .current_dir(&dir)
        .assert()
        .success();

    // Verify
    let adr_path = dir.path().join("doc/adr/0002-test-decision.md");
    assert!(adr_path.exists());
}
```

## Using tempdir

Always use `tempfile::tempdir()` for file-based tests:

```rust
use tempfile::tempdir;

#[test]
fn test_with_tempdir() {
    let dir = tempdir().unwrap();

    // dir.path() gives the path
    // Directory is automatically cleaned up when `dir` is dropped

    let file = dir.path().join("test.txt");
    std::fs::write(&file, "content").unwrap();
}
```

## Testing Both Modes

```rust
#[test]
fn test_compatible_mode() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path(), None, false).unwrap();
    // Test compatible behavior
}

#[test]
fn test_nextgen_mode() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path(), None, true).unwrap();
    // Test nextgen behavior
}
```

## See Also

- [Unit Tests](./unit-tests.md)
- [Property-Based Tests](./property-based-tests.md)
- [Fixtures](../fixtures.md)
