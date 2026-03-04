# Nygard Format

The Nygard format is the classic ADR structure from [adr-tools](https://github.com/npryce/adr-tools), based on Michael Nygard's [original blog post](http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions).

## Structure

```markdown
# 1. Record architecture decisions

Date: 2024-01-15

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as described by Michael Nygard.

## Consequences

See Michael Nygard's article for a detailed description.
```

## Sections

| Section | Required | Description |
|---------|----------|-------------|
| Title | Yes | Number and title as H1 heading |
| Date | Yes | When the decision was made |
| Status | Yes | Proposed, Accepted, Deprecated, Superseded |
| Context | Yes | Why is this decision needed? |
| Decision | Yes | What was decided? |
| Consequences | Yes | What are the implications? |

## Status Values

The Status section contains the current state of the decision:

- **Proposed**: Under discussion
- **Accepted**: Approved and in effect
- **Deprecated**: No longer recommended
- **Superseded**: Replaced by another ADR

### Links in Status

In Compatible mode, links to related ADRs appear in the Status section:

```markdown
## Status

Accepted

Supersedes [2. Use MySQL](0002-use-mysql.md)
Amended by [5. Add read replicas](0005-add-read-replicas.md)
```

## Template Variables

When creating custom templates, these variables are available:

| Variable | Description | Example |
|----------|-------------|---------|
| `number` | ADR number | `1` |
| `title` | ADR title | `Use PostgreSQL` |
| `date` | Current date | `2024-01-15` |
| `status` | Initial status | `Proposed` |

## Full Template

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

## Minimal Template

```markdown
# {{ number }}. {{ title }}

Date: {{ date }}

## Status

{{ status }}

## Context

## Decision

## Consequences
```

## When to Use Nygard Format

- Migrating from adr-tools
- Working with teams already using adr-tools
- Need shell scripts to parse ADRs
- Want maximum simplicity
- Don't need MADR metadata fields

## Usage

```sh
# Explicit format selection
adrs new --format nygard "My Decision"

# With variant
adrs new --format nygard --variant minimal "My Decision"
```

## Related

- [MADR Format](./madr.md) - Alternative format with more structure
- [Variants](./variants.md) - Full, minimal, bare variants
- [Compatible Mode](../modes/compatible.md) - Mode that uses Nygard by default
