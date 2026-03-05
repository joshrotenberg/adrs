# Frontmatter

In [NextGen mode](./modes/nextgen.md), ADRs use YAML frontmatter to store structured metadata. This page documents all available frontmatter fields.

## Basic Structure

Frontmatter is delimited by `---` markers:

```markdown
---
number: 1
title: Use PostgreSQL
date: 2024-01-15
status: accepted
---

# 1. Use PostgreSQL

...
```

## Field Reference

### Core Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `number` | integer | No | ADR number (can be inferred from filename) |
| `title` | string | No | ADR title (can be inferred from H1) |
| `date` | date | Yes | Decision date (YYYY-MM-DD) |
| `status` | string | Yes | Current status |

### Status Values

| Status | Description |
|--------|-------------|
| `proposed` | Under discussion |
| `accepted` | Approved and in effect |
| `deprecated` | No longer recommended |
| `superseded` | Replaced by another ADR |

### MADR Fields

| Field | Type | Description |
|-------|------|-------------|
| `decision-makers` | list | People who made the decision |
| `consulted` | list | People whose opinions were sought |
| `informed` | list | People who were informed |

```yaml
---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
  - Dave
informed:
  - Engineering Team
---
```

### Links

The `links` field stores relationships to other ADRs:

```yaml
---
links:
  - target: 2
    kind: supersedes
  - target: 3
    kind: amends
---
```

#### Link Fields

| Field | Type | Description |
|-------|------|-------------|
| `target` | integer | Target ADR number |
| `kind` | string | Relationship type |
| `description` | string | Optional description |

#### Link Kinds

| Kind | Reverse | Description |
|------|---------|-------------|
| `supersedes` | `superseded-by` | Replaces another ADR |
| `amends` | `amended-by` | Modifies another ADR |
| `related` | `related` | General relationship |

### Tags

```yaml
---
tags:
  - database
  - infrastructure
  - postgresql
---
```

### Custom Fields

You can add custom fields for your project's needs:

```yaml
---
status: accepted
date: 2024-01-15
team: platform
priority: high
jira: PROJ-123
---
```

## Complete Example

```yaml
---
number: 5
title: Use PostgreSQL for Persistence
date: 2024-01-15
status: accepted
decision-makers:
  - Alice (Tech Lead)
  - Bob (DBA)
consulted:
  - Carol (Security)
  - Dave (Ops)
informed:
  - Engineering Team
links:
  - target: 2
    kind: supersedes
    description: Replaces MySQL decision
  - target: 3
    kind: amends
tags:
  - database
  - infrastructure
---

# 5. Use PostgreSQL for Persistence

## Context and Problem Statement

...
```

## Parsing Behavior

- Frontmatter is optional in Compatible mode
- Frontmatter is expected in NextGen mode
- The parser auto-detects format
- Invalid YAML produces clear error messages
- Unknown fields are preserved (not stripped)

## Validation

The `adrs doctor` command validates frontmatter:

```sh
adrs doctor
```

Checks include:

- Required fields present
- Valid status value
- Valid date format
- Link targets exist
- No duplicate numbers

## Template Usage

In templates, frontmatter fields are available as variables:

```markdown
---
status: {{ status | lower }}
date: {{ date }}
{% if decision_makers %}
decision-makers:
{% for person in decision_makers %}
  - {{ person }}
{% endfor %}
{% endif %}
---
```

## Related

- [NextGen Mode](./modes/nextgen.md) - Mode that uses frontmatter
- [MADR Format](../reference/templates/madr.md) - Template format with full frontmatter
- [Templates](../reference/templates/README.md) - Template variable reference

> **Related:** [ADR-0006: YAML Frontmatter for Metadata](../reference/adrs/0006-yaml-frontmatter-for-metadata.md)
