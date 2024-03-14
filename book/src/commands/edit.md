# edit

## Overview

`edit` opens an ADR in your VISUAL/EDITOR that matches the given NAME argument.

## Help

```sh
Edit an existing Architectural Decision Record

Usage: adrs edit <NAME>

Arguments:
  <NAME>  The number of the ADR to edit

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

```sh
# find and edit the first ADR 
adrs edit 1  # looks for 0001-...
adrs edit data # fuzzy match for filenames with the string data in them
```

## Issues

See the [cmd-edit](https://github.com/joshrotenberg/adrs/labels/cmd-edit) label for command specific issues.
