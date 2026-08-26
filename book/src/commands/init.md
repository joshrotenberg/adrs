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

1. A configuration file in the current directory: `.adr-dir` (containing the
   ADR directory path) in compatible mode, or `adrs.toml` in NextGen mode
   (`--ng`). `init` does not write `.adrs.toml`; create that file or rename
   `adrs.toml` if you want a hidden config. An existing `.adrs.toml` is
   reused the same way as `adrs.toml`.
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

Writes `adrs.toml`, not `.adr-dir` or `.adrs.toml`, and creates the initial
ADR with YAML frontmatter:

```markdown
---
number: 1
title: Record architecture decisions
date: 2024-01-15
status: accepted
---

# 1. Record architecture decisions

...
```

## Re-initialization

`adrs init` is idempotent: running it again in an already-initialized
repository succeeds and preserves existing ADRs.

If `adrs.toml`, `.adrs.toml`, or `.adr-dir` is already present and its ADR
directory matches the directory being initialized, that file is left
unchanged. `--ng` does not convert `.adr-dir` or `.adrs.toml` into
`adrs.toml` on that path. Pass a different directory argument when you
intend to write a new config (that write also removes the other config
filenames so only one is authoritative).

The initial "Record architecture decisions" ADR is only created when the
repository has no ADRs yet, so re-initializing will not add a duplicate.

## Related

- [config](./config.md) - Show current configuration
- [new](./new.md) - Create additional ADRs
