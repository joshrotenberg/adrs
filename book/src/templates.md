# Templates

`adrs` uses templates to generate new ADR documents. You can use built-in templates or create custom ones.

## Built-in Templates

### Formats

| Format | Description |
|--------|-------------|
| `nygard` | Classic adr-tools format (default) |
| `madr` | MADR 4.0.0 format |

### Variants

Each format has three variants:

| Variant | Description |
|---------|-------------|
| `full` | Complete template with all sections and guidance comments |
| `minimal` | Essential sections only, no guidance |
| `bare` | Minimal structure with placeholders |

### Using Built-in Templates

```sh
# Default: nygard format, full variant
adrs new "My Decision"

# MADR format, full variant
adrs new --format madr "My Decision"

# Nygard format, minimal variant
adrs new --variant minimal "My Decision"

# MADR format, bare variant
adrs new --format madr --variant bare "My Decision"
```

### Template Examples

#### Nygard Full

```markdown
# {{ number }}. {{ title }}

Date: {{ date }}

## Status

{{ status }}

## Context

<!-- Describe the issue motivating this decision and any context -->

## Decision

<!-- What is the change that we're proposing and/or doing? -->

## Consequences

<!-- What becomes easier or more difficult to do because of this change? -->
```

#### MADR Full

```markdown
---
status: {{ status | lower }}
date: {{ date }}
---

# {{ title }}

## Context and Problem Statement

<!-- Describe the context and problem statement -->

## Decision Drivers

<!-- List the decision drivers -->

## Considered Options

<!-- List the options considered -->

## Decision Outcome

Chosen option: "", because ...

### Consequences

* Good, because ...
* Bad, because ...
```

## Custom Templates

Create custom templates using [Jinja2](https://jinja.palletsprojects.com/) syntax.

### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `number` | ADR number (padded) | `0001` |
| `title` | ADR title | `Use PostgreSQL` |
| `date` | Current date | `2024-01-15` |
| `status` | Initial status | `Proposed` |
| `links` | Array of links | See below |

### Link Variables

Each link in the `links` array has:

| Variable | Description |
|----------|-------------|
| `link.kind` | Link type (e.g., "Supersedes") |
| `link.target` | Target ADR number |
| `link.description` | Optional description |

### Creating a Custom Template

1. Create a template file:

```markdown
# {{ number }}. {{ title }}

**Date**: {{ date }}
**Status**: {{ status }}
**Author**: [Your Name]

## Problem

<!-- What problem does this solve? -->

## Solution

<!-- What is the solution? -->

## Alternatives Considered

<!-- What alternatives were considered? -->

## References

<!-- Any relevant links or documents -->
```

2. Use it when creating ADRs:

```sh
adrs new --template path/to/template.md "My Decision"
```

Or configure it in `adrs.toml`:

```toml
[templates]
custom = "templates/my-template.md"
```

### Conditional Sections

Use Jinja2 conditionals for optional content:

```markdown
{% if links %}
## Related Decisions

{% for link in links %}
* {{ link.kind }} [ADR {{ link.target }}]({{ link.target | pad(4) }}-*.md)
{% endfor %}
{% endif %}
```

### Filters

Available filters:

| Filter | Description | Example |
|--------|-------------|---------|
| `lower` | Lowercase | `{{ status \| lower }}` |
| `upper` | Uppercase | `{{ title \| upper }}` |
| `pad(n)` | Zero-pad number | `{{ number \| pad(4) }}` |

## Default Configuration

Set defaults in `adrs.toml`:

```toml
[templates]
# Default format for new ADRs
format = "madr"

# Default variant
variant = "minimal"

# Custom template path (optional)
# custom = "templates/custom.md"
```

## NextGen Mode Templates

When using `--ng` flag or `mode = "nextgen"`, templates generate YAML frontmatter:

```markdown
---
status: proposed
date: 2024-01-15
links:
  - kind: Supersedes
    target: 1
---

# Use PostgreSQL for Persistence

...
```
