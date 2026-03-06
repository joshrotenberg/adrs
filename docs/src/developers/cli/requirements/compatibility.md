# Compatibility Requirements

<!-- toc -->

## adr-tools Compatibility

### CLI-COMPAT-1: Command Compatibility

**Requirements:**
- MUST accept adr-tools command syntax
- MUST produce compatible file output in Compatible mode
- MUST read existing adr-tools repositories

### CLI-COMPAT-2: File Format

**Compatible mode output MUST match adr-tools:**

```markdown
# N. Title

Date: YYYY-MM-DD

## Status

Status

## Context

...

## Decision

...

## Consequences

...
```

### CLI-COMPAT-3: Configuration

**Requirements:**
- MUST read `.adr-dir` files
- MUST respect directory path from `.adr-dir`

### CLI-COMPAT-4: Numbering

**Requirements:**
- MUST use 4-digit zero-padded numbers (0001, 0002, ...)
- MUST auto-increment from highest existing number

### CLI-COMPAT-5: File Naming

**Requirements:**
- MUST use pattern: `NNNN-title-in-kebab-case.md`
- MUST lowercase titles
- MUST replace spaces with hyphens
- MUST remove special characters

## Cross-Platform

### CLI-COMPAT-6: Platform Support

**Requirements:**
- MUST work on Linux, macOS, Windows
- MUST handle path separators correctly
- MUST use appropriate line endings

### CLI-COMPAT-7: Encoding

**Requirements:**
- MUST use UTF-8 encoding
- MUST handle Unicode in titles

## See Also

- [Command Requirements](./commands.md)
- [Compatible Mode](../../../users/modes/compatible.md)
