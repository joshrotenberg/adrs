# Testing Guide

<!-- toc -->

This guide covers testing strategies and practices for the `adrs` project.

## Test Organization

```
crates/
├── adrs-core/
│   ├── src/           # Unit tests in modules
│   └── tests/         # Integration tests
│       ├── fixtures/  # Test ADR files
│       └── *.rs       # Test modules
├── adrs-cli/
│   └── tests/         # CLI integration tests
└── adrs-mcp/
    └── tests/         # MCP server tests
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

# Run tests matching pattern
cargo test parse::
```

## Coverage

Generate coverage reports:

```sh
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out html

# Generate lcov for CI
cargo tarpaulin --out lcov
```

## Best Practices

1. **Use tempdir**: Create temporary directories for file-based tests
2. **Test both modes**: Test Compatible and NextGen modes
3. **Test edge cases**: Empty files, malformed content, missing fields
4. **Clear assertions**: Use descriptive assertion messages
5. **Isolate tests**: No shared state between tests
6. **Fast tests**: Keep unit tests fast, use integration tests for I/O

## Documentation

- [Test Types](./types/README.md) - Unit, integration, property-based tests
- [Fixtures](./fixtures.md) - Test data management
- [Goals](./goals.md) - Coverage and quality goals

## See Also

- [Contributing](../contributing.md) - Testing requirements for PRs
