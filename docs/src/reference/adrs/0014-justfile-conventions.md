# 14. Justfile Conventions

Date: 2026-03-05

## Status

Accepted

## Context

The project uses [just](https://github.com/casey/just) as a command runner for build, test,
and development workflows. As the project grows, we need consistent conventions for
organizing justfiles across the project.

This ADR serves as the entrypoint for justfile conventions, with detailed decisions
documented in focused ADRs.

## Decision

Adopt consistent justfile conventions across the project, organized into the following areas:

### Convention ADRs

| ADR | Topic | Summary |
|-----|-------|---------|
| [0016](./0016-justfile-module-organization.md) | Module Organization | Hierarchical structure matching crate layout |
| [0017](./0017-justfile-global-settings.md) | Global Settings | Shell, tempdir, and working directory |
| [0018](./0018-justfile-recipe-conventions.md) | Recipe Conventions | Defaults, parameters, platforms, confirmations |
| [0019](./0019-justfile-argument-attributes.md) | Argument Attributes | Flag-style arguments with `[arg()]` |

### Related ADRs

| ADR | Topic | Relationship |
|-----|-------|--------------|
| [0015](./0015-visual-snapshot-testing.md) | Visual/Snapshot Testing | Test infrastructure used by justfile recipes |

### Version Requirements

| Feature | Minimum Version | ADR |
|---------|-----------------|-----|
| Modules | <sup>1.19.0</sup> | 0016 |
| `set shell` | <sup>0.5.0</sup> | 0017 |
| `set working-directory` | <sup>1.9.0</sup> | 0017 |
| `set tempdir` | <sup>1.10.0</sup> | 0017 |
| `[confirm]` | <sup>1.17.0</sup> | 0018 |
| `[default]` | <sup>1.42.0</sup> | 0018 |
| `[arg()]` | <sup>1.45.0</sup> | 0019 |
| Platform attributes | <sup>0.5.4</sup> | 0018 |

**Recommended minimum:** just 1.45.0 for full feature support.

## Consequences

See individual ADRs for specific consequences. General trade-offs:

### Positive

- Consistent patterns across all justfiles
- Self-documenting with module descriptions
- Reduced cognitive load for contributors

### Negative

- Requires relatively recent just version (1.45+) for all features
- Learning curve for contributors unfamiliar with just conventions

### Neutral

- One-time migration cost from existing patterns

## References

- [just manual](https://just.systems/man/en/)
- [just changelog](https://github.com/casey/just/blob/master/CHANGELOG.md)
