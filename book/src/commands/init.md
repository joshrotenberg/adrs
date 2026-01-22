# init

Initialize a new ADR repository.

## Usage

```
adrs init [OPTIONS] [DIRECTORY]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[DIRECTORY]` | ADR directory path (default: `doc/adr`) |

## Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode (YAML frontmatter) |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

The `init` command creates:

1. A `.adr-dir` file in the current directory containing the ADR directory path
2. The ADR directory (creates parent directories if needed)
3. An initial ADR: `0001-record-architecture-decisions.md`

## Examples

### Basic Initialization

```sh
adrs init
```

Creates:
```
.adr-dir          # Contains "doc/adr"
doc/
  adr/
    0001-record-architecture-decisions.md
```

### Custom Directory

```sh
adrs init decisions
```

Creates:
```
.adr-dir          # Contains "decisions"
decisions/
  0001-record-architecture-decisions.md
```

### Nested Directory

```sh
adrs init docs/architecture/decisions
```

Creates the full directory path.

### NextGen Mode

```sh
adrs init --ng
```

Creates the initial ADR with YAML frontmatter:

```markdown
---
status: accepted
date: 2024-01-15
---

# Record Architecture Decisions

...
```

## Error Handling

If the repository is already initialized (`.adr-dir` exists), the command will fail:

```
Error: ADR repository already initialized
```

To reinitialize, remove the `.adr-dir` file first.

## Related

- [config](./config.md) - Show current configuration
- [new](./new.md) - Create additional ADRs
