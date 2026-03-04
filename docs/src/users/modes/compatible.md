# Compatible Mode

Compatible mode is the default mode in `adrs`, providing full compatibility with [adr-tools](https://github.com/npryce/adr-tools).

## Overview

Compatible mode ensures that:

- Existing adr-tools workflows continue to work
- Shell scripts parsing ADRs don't break
- Teams can migrate gradually

## Configuration

Compatible mode uses `.adr-dir` for configuration:

```
doc/adr
```

This single-line file specifies the ADR directory path relative to the repository root.

## File Format

ADRs in Compatible mode use plain markdown without YAML frontmatter:

```markdown
# 1. Use PostgreSQL

Date: 2024-01-15

## Status

Accepted

Supersedes [2. Use MySQL](0002-use-mysql.md)

## Context

We need a database for our application.

## Decision

We will use PostgreSQL.

## Consequences

We need to learn PostgreSQL administration.
```

### Key Characteristics

| Aspect | Compatible Mode |
|--------|-----------------|
| Config file | `.adr-dir` |
| Status storage | `## Status` section |
| Links | Markdown links in Status section |
| Date | `Date: YYYY-MM-DD` inline |
| MADR fields | Not available |

## Status Section

The Status section contains:

1. The current status (Proposed, Accepted, Deprecated, Superseded)
2. Links to related ADRs

```markdown
## Status

Accepted

Supersedes [2. Use MySQL](0002-use-mysql.md)
Amended by [5. Add read replicas](0005-add-read-replicas.md)
```

## Link Types

Links in the Status section follow this pattern:

- `Supersedes [N. Title](path.md)`
- `Superseded by [N. Title](path.md)`
- `Amends [N. Title](path.md)`
- `Amended by [N. Title](path.md)`

## When to Use Compatible Mode

- Working with teams using adr-tools
- Need shell scripts to parse ADRs (grep for `## Status`)
- Want maximum simplicity
- Migrating from an existing adr-tools repository
- Don't need MADR metadata fields

## Limitations

Compatible mode does not support:

- YAML frontmatter
- MADR fields (`decision-makers`, `consulted`, `informed`)
- Structured link metadata
- Machine-readable status

## Migration to NextGen

To migrate to NextGen mode:

1. Create `adrs.toml`:

```toml
adr_dir = "doc/adr"
mode = "nextgen"
```

2. Existing files remain readable (parser auto-detects)
3. New ADRs will use frontmatter format
4. Optionally update existing ADRs manually

> **Note**: There is no automated migration tool yet. The parser reads both formats, so mixed repositories work for reading.

## Initialization

```sh
# Initialize in Compatible mode (default)
adrs init

# Creates .adr-dir file
```

## Related

- [NextGen Mode](./nextgen.md) - Enhanced features mode
- [Modes Overview](./README.md) - Mode comparison
- [Nygard Format](../../reference/templates/nygard.md) - Default template format

> **Related:** [ADR-0005: Dual Mode Operation](../../reference/adrs/0005-dual-mode-compatible-and-nextgen.md)
