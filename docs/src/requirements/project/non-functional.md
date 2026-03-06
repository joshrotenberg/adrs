# Non-Functional Requirements

<!-- toc -->

## Performance

### NFR-1: Speed

| Operation | Target | Maximum |
|-----------|--------|---------|
| List 1000 ADRs | < 500ms | < 1s |
| Parse single ADR | < 5ms | < 10ms |
| CLI startup | < 50ms | < 100ms |
| Search 1000 ADRs | < 200ms | < 500ms |

### NFR-2: Memory

| Scenario | Target | Maximum |
|----------|--------|---------|
| Idle CLI | < 10MB | < 20MB |
| List 1000 ADRs | < 50MB | < 100MB |
| MCP server idle | < 30MB | < 50MB |

## Reliability

### NFR-3: Error Handling

- Clear, actionable error messages
- No panics in library code
- Graceful degradation on partial failures
- No data corruption on errors

### NFR-4: Data Integrity

- Atomic file operations
- Backup before destructive operations
- Validate before write

## Compatibility

### NFR-5: Cross-Platform

| Platform | Support Level |
|----------|---------------|
| Linux (x64, arm64) | Full |
| macOS (x64, arm64) | Full |
| Windows (x64) | Full |

### NFR-6: Installation Methods

- Pre-built binaries
- Cargo install
- Homebrew (macOS)
- Shell installer

## Maintainability

### NFR-7: Architecture

- Library-first design
- Clear module boundaries
- Documented public API

### NFR-8: Code Quality

- No Clippy warnings
- Formatted with rustfmt
- Test coverage > 70%

### NFR-9: Documentation

- All public APIs documented
- User guide (this book)
- API reference on docs.rs

## Usability

### NFR-10: CLI Experience

- Consistent command syntax
- Helpful error messages
- Shell completions
- Reasonable defaults

### NFR-11: Configuration

- Sensible defaults
- Minimal required configuration
- Configuration discovery

## Security

### NFR-12: Safe Defaults

- No network access by default
- Operations scoped to repository
- Validate all inputs

### NFR-13: Dependency Management

- Minimal dependencies
- Audited dependencies
- Regular security updates

## See Also

- [Functional Requirements](./functional.md)
- [Testing Goals](../../developers/testing/goals.md)
