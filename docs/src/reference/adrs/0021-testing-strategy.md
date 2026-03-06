# 21. Testing Strategy

Date: 2026-03-05

## Status

Proposed

## Context

The project needs a consistent testing strategy that:

- Defines test types and their purposes
- Establishes where each test type lives
- Clarifies coverage expectations
- Guides contributors on what tests to write

Currently, testing practices are documented in `docs/src/developers/testing/` but not
formalized as architectural decisions.

## Decision

Adopt a multi-level testing strategy with the following test types:

### Test Type Hierarchy

| Level | Type | Purpose | Location | Runner |
|-------|------|---------|----------|--------|
| 1 | **Unit** | Test individual functions/methods in isolation | `src/**/*.rs` (inline) | `cargo test` |
| 2 | **Doctests** | Verify documentation examples work | `src/**/*.rs` (doc comments) | `cargo test --doc` |
| 3 | **Integration** | Test module interactions | `tests/*.rs` | `cargo test` |
| 4 | **Scenario** | Test realistic workflows | `tests/scenarios.rs` | `cargo test` |
| 5 | **Acceptance** | Test CLI commands end-to-end | `tests/cli_*.rs` | `cargo test` |
| 6 | **Smoke** | Quick sanity checks | `tests/justfile` | `just test smoke` |
| 7 | **Visual** | Output format regression | `tests/visual/` | `just test visual` |

### Level 1: Unit Tests

**Purpose:** Test individual functions in isolation.

**Characteristics:**
- Fast (< 100ms each)
- No filesystem I/O (use mocks or in-memory)
- No network
- Test one thing per test

**Location:** Inline `#[cfg(test)]` modules:

```rust
// src/parse.rs
pub fn parse_title(content: &str) -> Option<String> { ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_h1_title() { ... }

    #[test]
    fn returns_none_for_empty() { ... }
}
```

### Level 2: Doctests

**Purpose:** Ensure documentation examples are correct and serve as API usage examples.

**Requirements:**
- All public functions SHOULD have `# Examples` in doc comments
- Complex functions SHOULD have `# Errors` and `# Panics` sections
- Examples MUST compile and pass

**Location:** Doc comments with triple-backtick code blocks:

```rust
/// Parse the title from ADR content.
///
/// # Examples
///
/// ```rust
/// use adrs_core::parse_title;
///
/// let content = "# 1. Use Rust\n\nWe decided...";
/// assert_eq!(parse_title(content), Some("Use Rust".to_string()));
/// ```
///
/// # Errors
///
/// Returns `None` if no title found.
pub fn parse_title(content: &str) -> Option<String> { ... }
```

### Level 3: Integration Tests

**Purpose:** Test interactions between modules.

**Characteristics:**
- May use filesystem (via `tempfile`)
- Test multiple components together
- Slower than unit tests (< 1s each)

**Location:** `crates/<crate>/tests/*.rs`

```rust
// tests/repository_config.rs
#[test]
fn repository_uses_config_mode() {
    let temp = TempDir::new().unwrap();
    // Test Repository + Config interaction
}
```

### Level 4: Scenario Tests

**Purpose:** Test realistic user workflows end-to-end.

**Characteristics:**
- Multi-step workflows
- Simulate actual user behavior
- May be slower (1-5s each)

**Location:** `crates/adrs/tests/scenarios.rs`

```rust
#[test]
fn full_adr_lifecycle() {
    // init → new → edit → status → link → list → export
}
```

### Level 5: Acceptance Tests

**Purpose:** Test CLI commands produce correct output.

**Characteristics:**
- Run actual binary
- Check exit codes and output
- Use `assert_cmd` crate

**Location:** `crates/adrs/tests/cli_*.rs`

```rust
#[test]
fn help_displays_all_commands() {
    Command::cargo_bin("adrs")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"));
}
```

### Level 6: Smoke Tests

**Purpose:** Quick verification that commands don't crash.

**Characteristics:**
- Fast (total < 30s)
- Just checks "it runs"
- Run before commits

**Location:** `tests/justfile`

```just
smoke:
    $ADRS --help > /dev/null && echo "✓"
    $ADRS init > /dev/null && echo "✓"
    # ...
```

### Level 7: Visual/Snapshot Tests

**Purpose:** Detect unintended output format changes.

**Characteristics:**
- Compare output against expected files
- Fail on any difference
- Require explicit update when intentional

**Location:** `tests/visual/<command>/<case>.txt`

See [ADR-0015](./0015-visual-snapshot-testing.md) for details.

### Property-Based Tests

**Purpose:** Test invariants across random inputs.

**When to use:**
- Parsing functions
- Serialization round-trips
- Mathematical properties

**Library:** `proptest`

```rust
proptest! {
    #[test]
    fn slug_never_has_consecutive_dashes(title: String) {
        let slug = slugify(&title);
        assert!(!slug.contains("--"));
    }
}
```

### Test Organization by Crate

| Crate | Unit | Doctest | Integration | Scenario | Acceptance |
|-------|------|---------|-------------|----------|------------|
| `adrs-core` | ✅ | ✅ | ✅ | — | — |
| `adrs` (CLI) | ✅ | — | ✅ | ✅ | ✅ |
| `adrs-mcp` | ✅ | ✅ | ✅ | — | — |

### Coverage Expectations

| Test Type | Coverage Goal | Enforcement |
|-----------|---------------|-------------|
| Unit | 80%+ line coverage | CI warning |
| Doctest | All public API | `#![deny(missing_docs)]` |
| Integration | Critical paths | Code review |
| Scenario | Main workflows | Manual check |
| Acceptance | All CLI commands | Checklist |
| Smoke | All commands run | CI gate |

### When to Write Each Type

| Scenario | Test Type(s) |
|----------|--------------|
| New function | Unit + Doctest |
| New public API | Unit + Doctest + Integration |
| New CLI command | Acceptance + Smoke + Visual |
| Bug fix | Unit (regression test) |
| Refactor | Existing tests should pass |
| New workflow | Scenario |

## Consequences

### Positive

- Clear guidance for contributors
- Consistent test organization
- Multiple safety nets (unit → smoke → visual)
- Documentation stays accurate via doctests

### Negative

- More test types to maintain
- Learning curve for new contributors
- Visual tests require manual updates

### Neutral

- Trade-off between test granularity and maintenance burden
- Some overlap between test levels is acceptable

## References

- [Testing Guide](../../developers/testing/README.md)
- [ADR-0015: Visual/Snapshot Testing](./0015-visual-snapshot-testing.md)
- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
