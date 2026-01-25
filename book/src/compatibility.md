# Compatibility

This page explains the differences between compatible mode (default) and NextGen mode (`--ng`), including what breaks adr-tools compatibility.

## Mode Comparison

| Aspect | Compatible (default) | NextGen (--ng) | Breaks adr-tools? |
|--------|---------------------|----------------|-------------------|
| **Config file** | `.adr-dir` | `adrs.toml` | Yes |
| **File format** | Plain markdown | YAML frontmatter | Yes |
| **Status storage** | `## Status` section | `status:` in YAML | Yes |
| **Link storage** | Markdown links in Status | `links:` YAML array | Yes |
| **Date storage** | `Date: YYYY-MM-DD` inline | `date:` in YAML | Yes |
| **MADR fields** | Not available | `decision-makers`, `consulted`, `informed` | N/A |

## Key Incompatibilities with adr-tools

1. **adr-tools cannot read --ng files** - YAML frontmatter breaks bash parsing
2. **adr-tools cannot write to --ng repos** - Creates files without frontmatter
3. **Mixed repos don't work** - adr-tools would create incompatible files
4. **Scripts break** - Anything grep'ing for `## Status\n\nAccepted` fails

## What Still Works

- `adrs` can **read both formats** regardless of mode (parser auto-detects)
- Filename format is the same (`NNNN-slug.md`)
- JSON-ADR export works from either format

## File Format Examples

### Compatible Mode

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

### NextGen Mode

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

## When to Use Each Mode

### Use Compatible Mode when:

- Working with teams using adr-tools
- Need shell scripts to parse ADRs
- Want maximum simplicity
- Migrating from an existing adr-tools repository

### Use NextGen Mode when:

- Want MADR 4.0.0 metadata (decision-makers, consulted, informed)
- Need machine-readable structured metadata
- Building tooling that consumes ADRs programmatically
- Starting a new project without adr-tools dependency

## Migration Notes

- Switching to `--ng` does NOT auto-migrate existing files
- The parser reads both formats, so mixed repos work for reading
- Writing/updating ADRs uses the configured mode
- No automated migration tool yet (potential future feature)

## Enabling NextGen Mode

### For a new repository

```bash
adrs init --ng
```

### For individual commands

```bash
adrs --ng new "My Decision"
```

### Via configuration

Create `adrs.toml` in your repository root:

```toml
adr_dir = "doc/adr"
mode = "nextgen"
```
