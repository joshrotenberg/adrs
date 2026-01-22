# 5. Dual-mode operation: Compatible and NextGen

Date: 2025-01-21

## Status

Accepted

## Context

A primary goal of adrs is to be a drop-in replacement for the original adr-tools. However, there are improvements we want to make that would break compatibility with adr-tools:

- YAML frontmatter for structured metadata
- Enhanced linking capabilities
- Custom template formats
- Richer status tracking

We need to support both users migrating from adr-tools (who need compatibility) and users who want the enhanced features.

## Decision

Implement two modes of operation:

1. **Compatible mode** (default): Full compatibility with adr-tools
   - Uses `.adr-dir` configuration file
   - Generates plain markdown without frontmatter
   - Parses legacy status section format
   - Maintains filename conventions (NNNN-slug.md)

2. **NextGen mode** (opt-in via `--ng` flag or `adrs.toml`):
   - Uses `adrs.toml` configuration file
   - Generates YAML frontmatter with structured metadata
   - Supports enhanced features (custom templates, richer links)
   - Still readable by tools that ignore frontmatter

The mode is determined by:
1. Explicit `--ng` flag on commands
2. Presence of `adrs.toml` with `mode = "ng"`
3. Default to compatible mode

## Consequences

- Existing adr-tools users can migrate without any changes to their ADRs
- Power users can opt into enhanced features when ready
- The codebase must handle both formats in parsing and generation
- Documentation must clearly explain both modes
- Migration path from compatible to nextgen mode should be provided
