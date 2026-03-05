# Compatibility Requirements

<!-- toc -->

## Format Compatibility

### LIB-COMPAT-1: Parsing

**Requirements:**
- MUST parse adr-tools format files
- MUST parse MADR 4.0.0 format files
- MUST auto-detect format based on content
- MUST handle malformed files gracefully (return errors, not panic)

### LIB-COMPAT-2: Generation

**Requirements:**
- MUST generate adr-tools compatible output in Compatible mode
- MUST generate YAML frontmatter in NextGen mode
- Generated files MUST be parseable by the same library

### LIB-COMPAT-3: Configuration

| Format | Compatible Mode | NextGen Mode |
|--------|-----------------|--------------|
| `.adr-dir` | Read/Write | Read only |
| `adrs.toml` | Read only | Read/Write |

**Requirements:**
- MUST read both configuration formats
- MUST write to appropriate format based on mode
- MUST prefer `adrs.toml` when both exist

## adr-tools Compatibility

### LIB-COMPAT-4: Directory Structure

**Requirements:**
- MUST use `doc/adr` as default directory
- MUST respect `.adr-dir` content
- MUST create same file naming pattern: `NNNN-title-slug.md`

### LIB-COMPAT-5: File Format

**Compatible mode output:**
```markdown
# N. Title

Date: YYYY-MM-DD

## Status

Status

Link lines...

## Context

...

## Decision

...

## Consequences

...
```

### LIB-COMPAT-6: Link Format

**Compatible mode:**
```markdown
## Status

Accepted

Supersedes [1. Old Decision](0001-old-decision.md)
```

**NextGen mode:**
```yaml
---
status: accepted
links:
  - target: 1
    kind: supersedes
---
```

## MADR 4.0.0 Compatibility

### LIB-COMPAT-7: MADR Fields

**Requirements:**
- MUST support `decision-makers` field
- MUST support `consulted` field
- MUST support `informed` field
- Fields MUST be optional (for non-MADR usage)

### LIB-COMPAT-8: MADR Sections

**Requirements:**
- MUST support "Context and Problem Statement" section
- MUST support "Decision Drivers" section
- MUST support "Considered Options" section
- MUST support "Decision Outcome" section

## JSON-ADR Compatibility

### LIB-COMPAT-9: JSON-ADR Format

**Requirements:**
- MUST support JSON-ADR version 1.0.0
- Export MUST include all required fields
- Import MUST handle missing optional fields

### LIB-COMPAT-10: Round-Trip

**Requirements:**
- Export then import MUST preserve all data
- Order of ADRs MAY change
- Formatting MAY change

## See Also

- [ADR-0005: Dual Mode Operation](../../../reference/adrs/0005-dual-mode-compatible-and-nextgen.md)
- [ADR-0006: YAML Frontmatter](../../../reference/adrs/0006-yaml-frontmatter-for-metadata.md)
- [Compatible Mode](../../../users/modes/compatible.md)
- [NextGen Mode](../../../users/modes/nextgen.md)
