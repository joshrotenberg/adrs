# 10. Error handling strategy

Date: 2025-03-04

## Status

Proposed

## Context

A library-first architecture (ADR-0004) requires careful error handling:

- Library code must return `Result` types, not panic
- Errors must be actionable for library consumers
- CLI presents errors appropriately to users
- Error context should aid debugging

Rust's error handling ecosystem offers several approaches:

1. **thiserror**: Derive macros for custom error types
2. **anyhow**: Dynamic error handling with context
3. **Manual impl**: Full control, more boilerplate

## Decision

Use `thiserror` for library error types because:

- Clear, specific error variants
- Automatic `Display` and `Error` impl via derive macros
- Works well with `?` operator
- Library consumers can match on specific errors

Error type hierarchy:
```rust
pub enum Error {
    AdrDirNotFound { path: PathBuf },
    AdrNotFound { number: u32 },
    AmbiguousAdr { numbers: Vec<u32> },
    InvalidFormat { message: String },
    MissingField { field: String },
    Io { source: std::io::Error },
    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

Guidelines:
- Library returns `Result<T>` on all fallible operations
- No `unwrap()` or `expect()` in library code paths
- CLI converts errors to user-friendly messages
- Error variants include context (paths, numbers, field names)

## Consequences

- Library consumers can handle specific error cases
- Errors are self-documenting via variant names
- Dependency on `thiserror` crate
- New error cases require updating the enum
- CLI must translate errors for end users
