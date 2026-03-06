# Error Requirements

<!-- toc -->

## General Requirements

### LIB-ERR-1: No Panics

- Library MUST NOT panic in any public function
- Library MUST NOT use `unwrap()` or `expect()` in public code paths
- All errors MUST be returned as `Result<T, Error>`

### LIB-ERR-2: Error Types

```rust
pub enum Error {
    NotFound(PathBuf),
    Parse { path: PathBuf, line: Option<usize>, message: String },
    Config(String),
    Io(std::io::Error),
    Template(String),
    InvalidNumber(u32),
    LinkNotFound { source: u32, target: u32 },
}
```

**Requirements:**
- Errors MUST be typed (not stringly-typed)
- Errors MUST include relevant context (paths, line numbers)
- Errors MUST implement `std::error::Error`

### LIB-ERR-3: Error Messages

**Requirements:**
- Messages MUST be user-friendly (not internal debug info)
- Messages MUST be actionable when possible
- Messages MUST NOT expose internal paths or implementation details

### LIB-ERR-4: Error Conversion

**Requirements:**
- MUST implement `From<std::io::Error>` for I/O operations
- SHOULD be compatible with `anyhow` and `thiserror`
- MUST support the `?` operator for ergonomic error handling

## Specific Errors

### LIB-ERR-5: NotFound

**When:**
- Configuration file doesn't exist
- ADR file doesn't exist (for operations requiring it)
- Repository not initialized

**Message format:**
```
No ADR configuration found at {path}
```

### LIB-ERR-6: Parse

**When:**
- YAML frontmatter is malformed
- Required fields are missing
- Date format is invalid

**Message format:**
```
Failed to parse {path}: {message}
Failed to parse {path}:{line}: {message}
```

### LIB-ERR-7: Config

**When:**
- Configuration file is malformed
- Invalid configuration values

**Message format:**
```
Invalid configuration: {message}
```

### LIB-ERR-8: LinkNotFound

**When:**
- Attempting to link to non-existent ADR

**Message format:**
```
Cannot link ADR {source} to ADR {target}: target not found
```

## Recovery

### LIB-ERR-9: Graceful Degradation

**Requirements:**
- Partial failures SHOULD NOT corrupt existing data
- Failed operations SHOULD be atomic (all or nothing)
- Repository state SHOULD remain consistent after errors

## See Also

- [Error Handling Documentation](../errors.md)
- [API Requirements](./api.md)
