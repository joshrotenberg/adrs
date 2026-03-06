# 23. Documentation Conventions

Date: 2026-03-05

## Status

Proposed

## Context

The project needs consistent documentation standards for Rust code:

- Public API should be self-documenting
- Examples should be tested (doctests)
- Error conditions should be documented
- Cross-references should aid navigation
- Lints should enforce compliance

ADR-0021 covers doctests as a *test type* but doesn't define documentation
standards. This ADR fills that gap.

## Decision

### Required Doc Comment Headers

All public items MUST have doc comments. The following headers are used
based on the item type:

| Header | When Required | Description |
|--------|---------------|-------------|
| `# Examples` | All public functions/methods | Working code demonstrating usage |
| `# Arguments` | Functions with 2+ params or non-obvious params | Parameter descriptions |
| `# Returns` | Non-obvious return values | What the function returns |
| `# Errors` | Functions returning `Result` | When/why errors occur |
| `# Panics` | Functions that can panic | Conditions causing panic |
| `# Safety` | Unsafe functions | Invariants caller must uphold |
| `# Platform Notes` | Platform-specific behavior | Differences across OS/arch |

### Header Requirements by Item Type

#### Structs and Enums

```rust
/// Brief one-line description.
///
/// Longer description if needed. Explain the purpose and typical usage.
///
/// # Examples
///
/// ```rust
/// use my_crate::MyStruct;
///
/// let instance = MyStruct::new();
/// ```
pub struct MyStruct { }
```

#### Functions and Methods

```rust
/// Brief one-line description.
///
/// Longer description explaining behavior, edge cases, performance.
///
/// # Arguments
///
/// * `path` - The file path to read from
/// * `options` - Configuration options (see [`Options`])
///
/// # Returns
///
/// Returns the parsed configuration. Returns `None` if the file
/// doesn't exist (this is not an error for optional configs).
///
/// # Errors
///
/// Returns an error if:
/// - The file exists but cannot be read (permissions)
/// - The file contents are not valid TOML
///
/// # Panics
///
/// Panics if `path` contains invalid UTF-8. Use [`try_load`] for
/// non-UTF-8 paths.
///
/// # Examples
///
/// ```rust
/// use my_crate::load_config;
///
/// let config = load_config("config.toml")?;
/// assert_eq!(config.name, "example");
/// # Ok::<(), my_crate::Error>(())
/// ```
pub fn load_config(path: &str, options: Options) -> Result<Config> { }
```

#### Enum Variants

```rust
/// Source of configuration.
pub enum ConfigSource {
    /// Loaded from environment variables.
    ///
    /// Environment variables have the highest priority and override
    /// all file-based configuration.
    Environment,

    /// Loaded from a specific file via `ADRS_CONFIG` env var.
    ///
    /// Unlike optional config files, this is explicit: if set but
    /// file not found, it's an error.
    EnvironmentConfig(PathBuf),
}
```

### Example Requirements

Examples MUST:

1. **Compile** - All code blocks are tested by `cargo test --doc`
2. **Run** - Use `# Ok::<(), Error>(())` for Result-returning examples
3. **Assert** - Include assertions demonstrating expected behavior
4. **Be minimal** - Show one concept per example

```rust
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use my_crate::slugify;
///
/// assert_eq!(slugify("Hello World"), "hello-world");
/// ```
///
/// Handles special characters:
///
/// ```rust
/// use my_crate::slugify;
///
/// assert_eq!(slugify("What's Up?"), "whats-up");
/// ```
```

#### Example Annotations

| Annotation | When to Use |
|------------|-------------|
| `rust` | Default, compiles and runs |
| `rust,ignore` | Requires external setup (filesystem, network) |
| `rust,no_run` | Compiles but don't run (side effects) |
| `rust,should_panic` | Expected to panic |
| `text` | Not Rust code (output examples) |

### Lint Configuration

Enable these lints in `lib.rs`:

```rust
//! Crate-level documentation here.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]
```

| Lint | Level | Purpose |
|------|-------|---------|
| `missing_docs` | deny | All public items must have docs |
| `broken_intra_doc_links` | deny | Links to types/functions must resolve |
| `private_intra_doc_links` | deny | Don't link to private items from public docs |
| `missing_crate_level_docs` | warn | Crate should have `//!` docs |

### Cross-References

Use intra-doc links for references:

```rust
/// Loads configuration per [`ADR-0020`](crate::docs::adr_0020).
///
/// Uses [`Config::load`] internally. See also [`ConfigSource`]
/// for tracking where values came from.
///
/// [`ADR-0020`]: https://example.com/adrs/0020
```

### Module-Level Documentation

Each module SHOULD have `//!` documentation:

```rust
//! Configuration loading and management.
//!
//! This module provides:
//! - [`Config`] - The main configuration struct
//! - [`ConfigSource`] - Tracks where config values came from
//! - [`providers`] - Figment providers for various sources
//!
//! # Architecture
//!
//! Configuration is loaded using [Figment](https://docs.rs/figment)
//! with layered precedence per [ADR-0020].
//!
//! # Examples
//!
//! ```rust
//! use adrs_core::Config;
//!
//! let config = Config::load(".")?;
//! # Ok::<(), adrs_core::Error>(())
//! ```
```

### Documentation Checklist

Before submitting a PR with new public API:

- [ ] All public items have doc comments
- [ ] Functions with `Result` have `# Errors` section
- [ ] Functions that can panic have `# Panics` section
- [ ] Examples compile and run (`cargo test --doc`)
- [ ] Cross-references use intra-doc links
- [ ] `cargo doc --no-deps` produces no warnings

### Private Item Documentation

Private items SHOULD have doc comments if:
- The logic is non-obvious
- Other maintainers will need to understand it
- It implements a complex algorithm

Private docs use `//` comments for simple explanations or `///` if
they might become public later.

## Consequences

### Positive

- Self-documenting public API
- Examples are always up-to-date (tested)
- Compile-time enforcement via lints
- IDE support (hover docs, autocomplete)
- Generated docs on docs.rs

### Negative

- More upfront effort writing docs
- Doc comments add to file length
- Breaking changes require doc updates

### Neutral

- Trade-off between comprehensive docs and maintainability
- Some judgment required for "non-obvious" threshold

## References

- [ADR-0021: Testing Strategy](./0021-testing-strategy.md) - Doctests as test type
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html)
- [Rustdoc Book](https://doc.rust-lang.org/rustdoc/)
- [RFC 1574: API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)
