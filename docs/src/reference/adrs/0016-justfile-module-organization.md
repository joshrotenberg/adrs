# 16. Justfile Module Organization

Date: 2026-03-05

## Status

Accepted

## Context

As the project grows, a single monolithic justfile becomes unwieldy. We need a way to
organize recipes that:

- Matches the project's crate structure
- Allows teams to own their module's recipes
- Provides clear navigation via `just --list`
- Scales with project complexity

Just supports modules since version 1.19.0, allowing justfiles to import other justfiles.

## Decision

Use hierarchical module structure matching crate organization.

### Directory Structure

```
justfile                        # Root: imports modules, common recipes
├── crates/justfile             # Build recipes
├── crates/adrs/justfile        # CLI-specific recipes
├── crates/adrs-core/justfile   # Library-specific recipes
├── tests/justfile              # Test recipes
├── docs/justfile               # Documentation recipes
└── quality/justfile            # Quality/lint recipes
```

### Module Declaration <sup>1.19.0</sup>

Root justfile imports modules with `mod`:

```just
# Workspace build recipes (debug, release)
mod build './crates/justfile'

# CLI binary recipes (run, demo, smoke tests)
mod cli './crates/adrs/justfile'

# Test recipes (unit, integration, visual)
mod test './tests/justfile'
```

### Module Descriptions

Place a comment on the line **above** the `mod` declaration to provide a description
that appears in `just --list` output:

```just
# Documentation recipes (mdbook, rustdoc)
mod docs './docs/justfile'
```

Output:
```
docs ...    # Documentation recipes (mdbook, rustdoc)
```

### Nested Modules

Modules can import sub-modules for deeper hierarchies:

```just
# crates/justfile
set working-directory := ".."

# CLI binary (adrs) recipes
mod cli './adrs/justfile'

# Core library (adrs-core) recipes
mod lib './adrs-core/justfile'
```

### Invocation Patterns

| Pattern | Description |
|---------|-------------|
| `just build` | Invoke default recipe in build module |
| `just build release` | Invoke release recipe in build module |
| `just build::release` | Explicit module path (same as above) |
| `just build --list` | List recipes in build module |

## Consequences

### Positive

- Clear module boundaries matching project structure
- Self-documenting with module descriptions in `just --list`
- Teams can own and maintain their module's recipes
- Scales to large projects with many recipes
- Reduces merge conflicts (changes isolated to module files)

### Negative

- Module paths must be maintained when reorganizing crates
- Relative paths in modules can be confusing (use `set working-directory`)
- Some features don't work across module boundaries (see [ADR-0019](./0019-justfile-argument-attributes.md))

### Neutral

- Slight increase in file count (one justfile per module)
- Contributors must understand module navigation

## References

- [just modules documentation](https://just.systems/man/en/modules.html)
- [ADR-0014: Justfile Conventions](./0014-justfile-conventions.md) (entrypoint)
