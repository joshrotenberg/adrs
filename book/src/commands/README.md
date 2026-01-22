# Commands

`adrs` provides commands for managing Architecture Decision Records.

## Overview

| Command | Description |
|---------|-------------|
| [init](./init.md) | Initialize a new ADR repository |
| [new](./new.md) | Create a new ADR |
| [edit](./edit.md) | Edit an existing ADR |
| [link](./link.md) | Link two ADRs together |
| [list](./list.md) | List all ADRs |
| [config](./config.md) | Show configuration |
| [doctor](./doctor.md) | Check repository health |
| [generate](./generate.md) | Generate documentation |

## Global Options

These options are available for all commands:

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode (YAML frontmatter) |
| `-C, --cwd <DIR>` | Run as if started in `<DIR>` |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## Usage

```
adrs [OPTIONS] <COMMAND>

Commands:
  init      Initialize a new ADR repository
  new       Create a new ADR
  edit      Edit an existing ADR
  list      List all ADRs
  link      Link two ADRs together
  config    Show configuration
  doctor    Check repository health
  generate  Generate documentation
  help      Print this message or the help of the given subcommand(s)

Options:
      --ng                 Use next-gen mode (YAML frontmatter, enhanced features)
  -C, --cwd <WORKING_DIR>  Working directory (defaults to current directory)
  -h, --help               Print help
  -V, --version            Print version
```
