# Command Requirements

<!-- toc -->

## Core Commands

### CLI-CMD-1: Command Set

| Command | Description | Priority |
|---------|-------------|----------|
| `init` | Initialize ADR repository | P0 |
| `new` | Create new ADR | P0 |
| `list` | List ADRs | P0 |
| `edit` | Edit existing ADR | P0 |
| `status` | Change ADR status | P0 |
| `link` | Link two ADRs | P0 |
| `search` | Search ADRs | P1 |
| `export` | Export to JSON-ADR | P1 |
| `import` | Import from JSON-ADR | P1 |
| `generate` | Generate TOC/graph | P1 |
| `doctor` | Health checks | P1 |
| `config` | Show configuration | P2 |
| `template` | Manage templates | P2 |
| `completions` | Shell completions | P2 |

### CLI-CMD-2: Global Options

```
--ng              Enable NextGen mode
-C, --cwd <DIR>   Change working directory
-h, --help        Show help
-V, --version     Show version
```

**Requirements:**
- `--ng` MUST override configuration mode
- `-C` MUST change working directory before command execution
- All commands MUST support `-h` and `-V`

### CLI-CMD-3: Output Formats

**Requirements:**
- Default output MUST be human-readable text
- `--json` SHOULD provide JSON output for scripting
- `--quiet` SHOULD minimize output

## Individual Command Requirements

### CLI-CMD-4: init

```
adrs init [OPTIONS]
```

**Options:**
- `--ng`: Initialize in NextGen mode
- `-d, --dir <DIR>`: ADR directory (default: doc/adr)

**Behavior:**
- MUST create configuration file
- MUST create ADR directory
- MUST create initial ADR (0001-record-architecture-decisions.md)
- MUST NOT overwrite existing configuration

### CLI-CMD-5: new

```
adrs new [OPTIONS] <TITLE>
```

**Options:**
- `-f, --format <FORMAT>`: Template format
- `-v, --variant <VARIANT>`: Template variant
- `--status <STATUS>`: Initial status
- `-s, --supersedes <N>`: ADR to supersede
- `-l, --link <LINK>`: Link specification
- `--no-edit`: Skip editor

**Behavior:**
- MUST auto-assign next number
- MUST open editor unless `--no-edit`
- MUST create bidirectional links for supersedes
- MUST support multiple `-l` flags

### CLI-CMD-6: list

```
adrs list [OPTIONS]
```

**Options:**
- `--status <STATUS>`: Filter by status
- `--json`: JSON output

**Behavior:**
- MUST list all ADRs sorted by number
- MUST show number, title, and status

### CLI-CMD-7: status

```
adrs status <NUMBER> <STATUS> [--by <N>]
```

**Behavior:**
- MUST update ADR status
- `--by` MUST create supersession link
- MUST validate ADR exists

## See Also

- [UX Requirements](./ux.md)
- [Commands Reference](../../../users/commands/README.md)
