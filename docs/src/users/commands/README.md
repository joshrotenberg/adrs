# Commands

`adrs` provides commands for managing Architecture Decision Records.

## Overview

| Command | Description |
|---------|-------------|
| [init](./init.md) | Initialize a new ADR repository |
| [new](./new.md) | Create a new ADR |
| [edit](./edit.md) | Edit an existing ADR |
| [list](./list.md) | List all ADRs |
| [search](./search.md) | Search ADRs for matching content |
| [status](./status.md) | Change an ADR's status |
| [link](./link.md) | Link two ADRs together |
| [export](./export.md) | Export ADRs to different formats |
| [import](./import.md) | Import ADRs from different formats |
| [generate](./generate.md) | Generate documentation |
| [template](./template.md) | Manage ADR templates |
| [config](./config.md) | Show configuration |
| [doctor](./doctor.md) | Check repository health |
| [completions](./completions.md) | Generate shell completions |
| [cheatsheet](./cheatsheet.md) | Show quick reference |

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
  init         Initialize a new ADR repository
  new          Create a new ADR
  edit         Edit an existing ADR
  list         List all ADRs
  search       Search ADRs for matching content
  link         Link two ADRs together
  status       Change an ADR's status
  config       Show configuration
  doctor       Check repository health
  generate     Generate documentation
  export       Export ADRs to different formats
  import       Import ADRs from different formats
  template     Manage ADR templates
  completions  Generate shell completions
  cheatsheet   Show quick reference for common workflows
  help         Print this message or the help of the given subcommand(s)

Options:
      --ng                 Use next-gen mode (YAML frontmatter, enhanced features)
  -C, --cwd <WORKING_DIR>  Working directory (defaults to current directory)
  -h, --help               Print help
  -V, --version            Print version
```
