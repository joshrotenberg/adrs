# Unit Tests

Unit tests verify individual functions and types in isolation.

## Location

Unit tests are located in source files with `#[cfg(test)]` modules:

```rust
// src/types.rs

pub fn generate_slug(title: &str) -> String {
    title.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_basic() {
        assert_eq!(generate_slug("Use PostgreSQL"), "use-postgresql");
    }

    #[test]
    fn test_generate_slug_special_chars() {
        assert_eq!(generate_slug("Use C++"), "use-c");
    }

    #[test]
    fn test_generate_slug_empty() {
        assert_eq!(generate_slug(""), "");
    }
}
```

## Best Practices

### Test One Thing

```rust
// Good: One assertion per logical concept
#[test]
fn test_adr_new_sets_proposed_status() {
    let adr = Adr::new(1, "Test");
    assert_eq!(adr.status, AdrStatus::Proposed);
}

#[test]
fn test_adr_new_sets_today_date() {
    let adr = Adr::new(1, "Test");
    assert_eq!(adr.date, today());
}
```

### Descriptive Names

```rust
// Good: Describes what is being tested
#[test]
fn test_parse_frontmatter_with_missing_status_returns_error() { }

// Bad: Too vague
#[test]
fn test_parse() { }
```

### Test Edge Cases

```rust
#[test]
fn test_slug_with_unicode() {
    assert_eq!(generate_slug("Café"), "caf");
}

#[test]
fn test_slug_with_numbers() {
    assert_eq!(generate_slug("Use HTTP/2"), "use-http2");
}

#[test]
fn test_slug_with_only_spaces() {
    assert_eq!(generate_slug("   "), "");
}
```

### Use Helper Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_adr() -> Adr {
        Adr::new(1, "Test ADR")
    }

    #[test]
    fn test_with_sample() {
        let adr = sample_adr();
        // ...
    }
}
```

## Running Unit Tests

```sh
# Run all unit tests
cargo test --lib

# Run unit tests in specific crate
cargo test -p adrs-core --lib

# Run specific test
cargo test types::tests::test_generate_slug
```

## See Also

- [Integration Tests](./integration-tests.md)
- [Test Types Overview](./README.md)
