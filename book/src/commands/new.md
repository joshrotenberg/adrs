# new

Create a new Architecture Decision Record.

## Usage

```
adrs new [OPTIONS] <TITLE>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<TITLE>` | Title of the new ADR |

## Options

| Option | Description |
|--------|-------------|
| `-f, --format <FORMAT>` | Template format: `nygard` or `madr` (default: `nygard`) |
| `-v, --variant <VARIANT>` | Template variant: `full`, `minimal`, or `bare` (default: `full`) |
| `--status <STATUS>` | Initial status (default: `Proposed`) |
| `-s, --supersedes <N>` | ADR number(s) this supersedes |
| `-l, --link <LINK>` | Link to another ADR |
| `--ng` | Use NextGen mode (YAML frontmatter) |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Creates a new ADR file with the next available number. Opens your `$EDITOR` to edit the document. The ADR is saved when you close the editor.

## Examples

### Basic Usage

```sh
adrs new "Use PostgreSQL for persistence"
```

Creates `0002-use-postgresql-for-persistence.md` and opens it in your editor.

### With MADR Format

```sh
adrs new --format madr "Use PostgreSQL for persistence"
```

### Minimal Template

```sh
adrs new --variant minimal "Use PostgreSQL for persistence"
```

### With Initial Status

```sh
adrs new --status Accepted "Use PostgreSQL for persistence"
```

### Superseding an ADR

```sh
adrs new --supersedes 2 "Use MySQL instead of PostgreSQL"
```

This creates a new ADR and:
- Adds a "Supersedes" link in the new ADR
- Adds a "Superseded by" link in ADR #2
- Changes ADR #2's status to "Superseded"

### Superseding Multiple ADRs

```sh
adrs new --supersedes 2 --supersedes 3 "Consolidated database decision"
```

### Linking to Another ADR

```sh
adrs new --link "2:Amends:Amended by" "Clarify database choice"
```

The link format is `TARGET:KIND:REVERSE_KIND`:
- `TARGET`: The ADR number to link to
- `KIND`: The relationship from this ADR to the target
- `REVERSE_KIND`: The reverse relationship added to the target

Common link types:
- `Amends` / `Amended by`
- `Extends` / `Extended by`
- `Relates to` / `Relates to`

### Multiple Links

```sh
adrs new --link "2:Amends:Amended by" --link "3:Extends:Extended by" "Combined decision"
```

### NextGen Mode with MADR

```sh
adrs new --ng --format madr "Use PostgreSQL for persistence"
```

Creates an ADR with YAML frontmatter:

```markdown
---
status: proposed
date: 2024-01-15
---

# Use PostgreSQL for Persistence

## Context and Problem Statement
...
```

## Editor

The `new` command uses the `$EDITOR` environment variable. If not set, it tries common editors:
- `vim`
- `nano`
- `vi`

Set your preferred editor:

```sh
export EDITOR="code --wait"  # VS Code
export EDITOR="nano"         # Nano
export EDITOR="vim"          # Vim
```

## File Naming

ADR files are named using the pattern:

```
NNNN-title-in-kebab-case.md
```

Where:
- `NNNN` is the zero-padded ADR number
- Title is converted to lowercase kebab-case
- Special characters are removed

Examples:
- `0001-record-architecture-decisions.md`
- `0002-use-postgresql-for-persistence.md`
- `0015-api-versioning-strategy.md`

## Related

- [edit](./edit.md) - Edit an existing ADR
- [link](./link.md) - Link ADRs together
- [Templates](../templates.md) - Available templates
