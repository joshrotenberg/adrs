# Testing Goals

Quality and coverage goals for the `adrs` project.

## Coverage Goals

| Crate | Target | Current |
|-------|--------|---------|
| adrs-core | 80% | - |
| adrs-cli | 70% | - |
| adrs-mcp | 70% | - |

### What to Cover

**High Priority:**
- Public API functions
- Error handling paths
- Edge cases in parsing
- Mode switching logic

**Medium Priority:**
- Helper functions
- Output formatting
- Configuration loading

**Lower Priority:**
- Generated code (clap derives)
- Simple getters/setters
- Debug implementations

## Quality Goals

### Reliability

- No panics in production code
- Graceful error handling
- No data corruption on failure

### Maintainability

- Tests document behavior
- Fast test execution
- Easy to add new tests

### Regression Prevention

- Bug fixes include tests
- Feature changes update tests
- CI blocks on test failures

## Test Characteristics

### FIRST Principles

- **Fast**: Unit tests < 1ms, integration < 100ms
- **Isolated**: No shared state, no test order dependencies
- **Repeatable**: Same result every time
- **Self-validating**: Pass/fail, no manual verification
- **Timely**: Written with or before code

### Test Quality Checks

- [ ] Tests have descriptive names
- [ ] Tests document expected behavior
- [ ] Tests cover happy path and error cases
- [ ] Tests are isolated (use tempdir)
- [ ] Tests are deterministic

## CI Integration

```yaml
# .github/workflows/ci.yml
- name: Run tests
  run: cargo test --all

- name: Check coverage
  run: |
    cargo tarpaulin --out xml
    # Upload to coverage service
```

## See Also

- [Testing Guide](./README.md)
- [Contributing](../contributing.md)
