# Property-Based Tests

Property-based tests verify invariants by generating random test cases.

## When to Use

- Testing invariants that should hold for any input
- Finding edge cases you didn't think of
- Testing parsing/serialization round-trips
- Verifying mathematical properties

## Using proptest

Add to `Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1.0"
```

## Example: Slug Generation

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_slug_never_contains_spaces(title in "[a-zA-Z ]+") {
        let slug = generate_slug(&title);
        prop_assert!(!slug.contains(' '));
    }

    #[test]
    fn test_slug_is_lowercase(title in "[a-zA-Z]+") {
        let slug = generate_slug(&title);
        prop_assert_eq!(slug, slug.to_lowercase());
    }
}
```

## Example: Round-Trip

```rust
use proptest::prelude::*;
use adrs_core::{Adr, AdrStatus};

fn arb_status() -> impl Strategy<Value = AdrStatus> {
    prop_oneof![
        Just(AdrStatus::Proposed),
        Just(AdrStatus::Accepted),
        Just(AdrStatus::Deprecated),
        Just(AdrStatus::Superseded),
        "[a-z]+".prop_map(AdrStatus::Custom),
    ]
}

proptest! {
    #[test]
    fn test_status_roundtrip(status in arb_status()) {
        let string = status.to_string();
        let parsed: AdrStatus = string.parse().unwrap();
        prop_assert_eq!(status, parsed);
    }
}
```

## Example: Parsing Invariants

```rust
proptest! {
    #[test]
    fn test_parse_never_panics(content in ".*") {
        let parser = Parser::new();
        // Should return Ok or Err, never panic
        let _ = parser.parse(&content);
    }
}
```

## Strategies

### Built-in Strategies

```rust
// Strings
"[a-z]+"           // Regex
any::<String>()    // Any string
".*".prop_map(|s| s.trim().to_string())

// Numbers
0..100u32          // Range
any::<u32>()       // Any u32

// Collections
prop::collection::vec(any::<String>(), 0..10)
```

### Custom Strategies

```rust
fn arb_adr() -> impl Strategy<Value = Adr> {
    (1..1000u32, "[A-Z][a-z ]+", arb_status())
        .prop_map(|(number, title, status)| {
            let mut adr = Adr::new(number, title);
            adr.status = status;
            adr
        })
}
```

## Running

```sh
# Run property tests
cargo test --test proptest_tests

# With more cases
PROPTEST_CASES=1000 cargo test
```

## See Also

- [proptest documentation](https://proptest-rs.github.io/proptest/)
- [Unit Tests](./unit-tests.md)
- [Integration Tests](./integration-tests.md)
