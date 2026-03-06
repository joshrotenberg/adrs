# NextGen Mode

NextGen mode enables enhanced features including YAML frontmatter, structured metadata, and full MADR 4.0.0 support.

## Overview

NextGen mode provides:

- YAML frontmatter for machine-readable metadata
- Full MADR 4.0.0 field support
- Structured link storage
- Better tooling integration

## Enabling NextGen Mode

### For a New Repository

```sh
adrs init --ng
```

This creates `adrs.toml` instead of `.adr-dir`.

### For Individual Commands

```sh
adrs --ng new "My Decision"
adrs --ng list
```

### Via Configuration

Create `adrs.toml` in your repository root:

```toml
adr_dir = "doc/adr"
mode = "nextgen"

[templates]
format = "madr"
variant = "minimal"
```

## File Format

ADRs in NextGen mode use YAML frontmatter:

```markdown
---
number: 1
title: Use PostgreSQL
date: 2024-01-15
status: accepted
links:
  - target: 2
    kind: supersedes
---

# 1. Use PostgreSQL

## Context

We need a database for our application.

## Decision

We will use PostgreSQL.

## Consequences

We need to learn PostgreSQL administration.
```

### Key Characteristics

| Aspect | NextGen Mode |
|--------|--------------|
| Config file | `adrs.toml` |
| Status storage | `status:` in YAML |
| Links | `links:` YAML array |
| Date | `date:` in YAML |
| MADR fields | Full support |

## YAML Frontmatter

See [Frontmatter](../frontmatter.md) for the complete field reference.

### Core Fields

```yaml
---
number: 1
title: Use PostgreSQL
date: 2024-01-15
status: accepted
---
```

### MADR Fields

```yaml
---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
informed:
  - Dave
---
```

### Links

```yaml
---
links:
  - target: 2
    kind: supersedes
  - target: 3
    kind: amends
---
```

## When to Use NextGen Mode

- Starting a new project without adr-tools dependency
- Want MADR 4.0.0 metadata (decision-makers, consulted, informed)
- Need machine-readable structured metadata
- Building tooling that consumes ADRs programmatically
- Want structured link tracking

## Incompatibilities with adr-tools

> **Warning**: NextGen mode is **not compatible** with adr-tools.

| Issue | Impact |
|-------|--------|
| YAML frontmatter | adr-tools cannot parse |
| No Status section | adr-tools scripts fail |
| Structured links | Different format |

## Interoperability

- `adrs` can read both Compatible and NextGen formats
- Mixed repositories work for **reading** only
- **Writing** always uses the configured mode
- adr-tools cannot read NextGen files

## Configuration Options

Full `adrs.toml` example:

```toml
# ADR directory (relative to repo root)
adr_dir = "doc/adr"

# Mode: "compatible" or "nextgen"
mode = "nextgen"

[templates]
# Default format: "nygard" or "madr"
format = "madr"

# Default variant: "full", "minimal", "bare", "bare-minimal"
variant = "minimal"

# Custom template path (optional)
# custom = "templates/my-template.md"
```

## Migration from Compatible Mode

1. Create `adrs.toml` (existing `.adr-dir` can remain):

```toml
adr_dir = "doc/adr"
mode = "nextgen"
```

2. The parser auto-detects file format
3. Existing ADRs remain readable
4. New ADRs use frontmatter format
5. Optionally convert existing ADRs manually

## Related

- [Compatible Mode](./compatible.md) - adr-tools compatible mode
- [Modes Overview](./README.md) - Mode comparison
- [Frontmatter](../frontmatter.md) - YAML field reference
- [MADR Format](../../reference/templates/madr.md) - Recommended template for NextGen

> **Related:** [ADR-0005: Dual Mode Operation](../../reference/adrs/0005-dual-mode-compatible-and-nextgen.md)
