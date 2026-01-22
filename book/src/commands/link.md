# link

Link two Architecture Decision Records together.

## Usage

```
adrs link [OPTIONS] <SOURCE> <LINK> <TARGET> <REVERSE_LINK>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<SOURCE>` | Source ADR number |
| `<LINK>` | Link description from source to target |
| `<TARGET>` | Target ADR number |
| `<REVERSE_LINK>` | Reverse link description from target to source |

## Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Creates bidirectional links between two ADRs. The link is added to the status section of both ADRs.

## Examples

### Amends Relationship

```sh
adrs link 3 "Amends" 1 "Amended by"
```

Result in ADR #3:
```markdown
## Status

Proposed

Amends [1. Record architecture decisions](0001-record-architecture-decisions.md)
```

Result in ADR #1:
```markdown
## Status

Accepted

Amended by [3. Clarify decision format](0003-clarify-decision-format.md)
```

### Extends Relationship

```sh
adrs link 4 "Extends" 2 "Extended by"
```

### Custom Relationship

```sh
adrs link 5 "Depends on" 3 "Dependency of"
```

## Common Link Types

| Forward Link | Reverse Link | Use Case |
|--------------|--------------|----------|
| Amends | Amended by | Modifying a previous decision |
| Extends | Extended by | Building on a previous decision |
| Supersedes | Superseded by | Replacing a decision |
| Relates to | Relates to | General relationship |
| Depends on | Dependency of | Dependencies |

## Superseding

For superseding relationships, prefer using `adrs new --supersedes`:

```sh
# Instead of:
adrs new "New approach"
adrs link 3 "Supersedes" 2 "Superseded by"

# Use:
adrs new --supersedes 2 "New approach"
```

The `--supersedes` option also updates the target ADR's status to "Superseded".

## NextGen Mode

In NextGen mode (`--ng`), links are stored in YAML frontmatter:

```yaml
---
status: proposed
date: 2024-01-15
links:
  - kind: Amends
    target: 1
---
```

## Related

- [new](./new.md) - Create ADRs with links
- [list](./list.md) - Find ADR numbers
