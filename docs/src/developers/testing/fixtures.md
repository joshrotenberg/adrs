# Test Fixtures

Test fixtures are pre-created files used for testing.

## Location

```
crates/adrs-core/tests/fixtures/
├── valid/
│   ├── 0001-simple-compatible.md
│   ├── 0002-with-links.md
│   ├── 0003-nextgen-format.md
│   └── 0004-madr-format.md
├── invalid/
│   ├── missing-status.md
│   ├── bad-frontmatter.md
│   └── malformed-links.md
└── repositories/
    ├── compatible/
    │   ├── .adr-dir
    │   └── doc/adr/
    └── nextgen/
        ├── adrs.toml
        └── doc/adr/
```

## Using Fixtures

### Reading Fixtures

```rust
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn test_parse_compatible_format() {
    let path = fixture_path("valid/0001-simple-compatible.md");
    let content = std::fs::read_to_string(&path).unwrap();

    let parser = Parser::new();
    let adr = parser.parse(&content).unwrap();

    assert_eq!(adr.number, 1);
}
```

### Copying Fixture Repositories

```rust
use std::fs;
use tempfile::tempdir;

fn copy_fixture_repo(name: &str) -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let fixture = fixture_path(&format!("repositories/{}", name));

    copy_dir_all(&fixture, dir.path()).unwrap();

    dir
}

#[test]
fn test_with_fixture_repo() {
    let dir = copy_fixture_repo("compatible");
    let repo = Repository::open(dir.path()).unwrap();

    let adrs = repo.list().unwrap();
    assert!(!adrs.is_empty());
}
```

## Creating Fixtures

### Valid Fixtures

Include a variety of valid files:

```markdown
<!-- 0001-simple-compatible.md -->
# 1. Simple Decision

Date: 2024-01-15

## Status

Accepted

## Context

Simple context.

## Decision

Simple decision.

## Consequences

Simple consequences.
```

### Invalid Fixtures

Include common error cases:

```markdown
<!-- missing-status.md -->
# 1. Missing Status

Date: 2024-01-15

## Context

This file has no status section.
```

### Edge Cases

Include boundary conditions:

```markdown
<!-- empty-sections.md -->
# 1. Empty Sections

Date: 2024-01-15

## Status

Proposed

## Context

## Decision

## Consequences
```

## Best Practices

1. **Organize by category**: valid/, invalid/, edge-cases/
2. **Document purpose**: Comment explaining what each fixture tests
3. **Keep minimal**: Only include what's needed for the test
4. **Version control**: Fixtures are part of the codebase
5. **Update with format changes**: Keep fixtures in sync with code

## See Also

- [Integration Tests](./types/integration-tests.md)
- [Test Types](./types/README.md)
