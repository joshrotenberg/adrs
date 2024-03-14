# init

## Overview

The `init` command initializes a new ADR directory, using `doc/adr` by default. An alternate directory can optionally be supplied to the command
to store ADRs in a different location. `init` will also create a `.adr-dir` file to store the directory location so that other commands
can find the top level directory.

`init` creates an initial ADR for you as well, noting the decision you've made to document your Architectural Decisions using ADRs.
Good job by you.

## Help

```sh
Initializes the directory of Architecture Decision Records

Usage: adrs init [DIRECTORY]

Arguments:
  [DIRECTORY]  Directory to initialize [default: doc/adr]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

```sh
# use the default location
adrs init

# put your ADRs somewhere else
adrs init some/other/place
```

## Issues

See the [cmd-init](https://github.com/joshrotenberg/adrs/labels/cmd-init) label for command specific issues.
