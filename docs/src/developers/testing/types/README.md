# Test Types

Overview of testing strategies used in the `adrs` project.

## Summary

| Type | Location | Purpose | Speed |
|------|----------|---------|-------|
| [Unit Tests](./unit-tests.md) | `src/` modules | Test individual functions | Fast |
| [Integration Tests](./integration-tests.md) | `tests/` directories | Test module interactions | Medium |
| [Property-Based Tests](./property-based-tests.md) | `tests/` with proptest | Test invariants | Slow |

## When to Use Each

### Unit Tests

- Testing pure functions
- Testing individual types
- Testing error conditions
- Fast feedback during development

### Integration Tests

- Testing file I/O
- Testing command execution
- Testing end-to-end workflows
- Verifying system behavior

### Property-Based Tests

- Testing invariants (e.g., "slug never contains spaces")
- Finding edge cases
- Testing with generated data
- Increasing confidence in parsing/serialization

## Test Pyramid

```
         /\
        /  \  E2E (few)
       /----\
      /      \  Integration (some)
     /--------\
    /          \  Unit (many)
   /------------\
```

- **Many unit tests**: Fast, focused, easy to maintain
- **Some integration tests**: Test interactions, slower
- **Few E2E tests**: Full workflow, slowest

## See Also

- [Testing Guide](../README.md) - Running tests
- [Fixtures](../fixtures.md) - Test data
