# Templates

`adrs` uses templates to generate new ADR documents. Templates define the structure and sections of your ADRs.

## Overview

Templates in `adrs` consist of two components:

- **Format**: The overall structure (Nygard or MADR)
- **Variant**: The level of detail (full, minimal, bare, bare-minimal)

## Available Formats

| Format | Description | Best For |
|--------|-------------|----------|
| [Nygard](./nygard.md) | Classic adr-tools format | Simple decisions, adr-tools migration |
| [MADR](./madr.md) | MADR 4.0.0 format | Detailed decisions, team tracking |

## Available Variants

| Variant | Description |
|---------|-------------|
| `full` | All sections with guidance comments |
| `minimal` | Essential sections only |
| `bare` | All sections, empty (no guidance) |
| `bare-minimal` | Core sections only, empty |

See [Variants](./variants.md) for detailed examples of each.

## Quick Start

```sh
# Default: Nygard format, full variant
adrs new "Use PostgreSQL"

# MADR format
adrs new --format madr "Use PostgreSQL"

# Minimal variant
adrs new --variant minimal "Use PostgreSQL"

# Combine format and variant
adrs new --format madr --variant bare "Use PostgreSQL"
```

## Choosing a Format

| Use Nygard when... | Use MADR when... |
|--------------------|------------------|
| Migrating from adr-tools | Starting a new project |
| Simplicity is preferred | You need structured metadata |
| Team is familiar with it | You want decision-maker tracking |
| | You need detailed option comparison |

## Custom Templates

You can create custom templates using Jinja2 syntax. See the individual format pages for template variable reference.

### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `number` | ADR number | `1` |
| `title` | ADR title | `Use PostgreSQL` |
| `date` | Current date | `2024-01-15` |
| `status` | Initial status | `Proposed` |
| `is_ng` | NextGen mode active | `true` |

### Configuration

Set defaults in `adrs.toml`:

```toml
[templates]
format = "madr"
variant = "minimal"

# Optional: custom template path
# custom = "templates/my-template.md"
```

## Related

- [Nygard Format](./nygard.md) - Classic ADR format details
- [MADR Format](./madr.md) - MADR 4.0.0 format details
- [Variants](./variants.md) - Template variant examples
- [Frontmatter](../../users/frontmatter.md) - YAML metadata in NextGen mode

> **Related:** [ADR-0007: Use minijinja for Templates](../adrs/0007-use-minijinja-for-templates.md)
