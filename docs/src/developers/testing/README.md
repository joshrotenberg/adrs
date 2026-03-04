# Testing Guide

This guide covers testing strategies and practices for the `adrs` project.

## Test Organization

```
crates/
├── adrs-core/
│   ├── src/           # Source code
│   └── tests/         # Integration tests
│       ├── fixtures/  # Test ADR files
│       └── *.rs       # Test modules
├── adrs-cli/
│   └── tests/         # CLI integration tests
└── adrs-mcp/
    └── tests/         # MCP server tests
```

## Test Types

### Unit Tests

Located in source files with `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Integration Tests

Located in `tests/` directories:

```rust
// tests/repository_tests.rs
use adrs_core::Repository;
use tempfile::tempdir;

#[test]
fn test_create_adr() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    // ...
}
```

### Property-Based Tests

Using `proptest` for generating test cases:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_slug_generation(title in "[a-zA-Z ]+") {
        let slug = generate_slug(&title);
        assert!(!slug.contains(' '));
    }
}
```

## Running Tests

```sh
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a specific crate
cargo test -p adrs-core
```

## Test Fixtures

Test fixtures are ADR files used for testing:

```
tests/fixtures/
├── valid/
│   ├── 0001-simple.md
│   └── 0002-with-links.md
└── invalid/
    ├── missing-status.md
    └── bad-frontmatter.md
```

## Coverage

Generate coverage reports:

```sh
cargo tarpaulin --out html
```

## Best Practices

1. **Use tempdir**: Create temporary directories for file-based tests
2. **Test both modes**: Test Compatible and NextGen modes
3. **Test edge cases**: Empty files, malformed content, missing fields
4. **Clean assertions**: Clear failure messages
5. **Isolate tests**: No shared state between tests
