# link

## Overview

The `link` command links together the SOURCE and TARGET ADRs.

## Help

```sh
Link Architectural Decision Records

Usage: adrs link <SOURCE> <LINK> <TARGET> <REVERSE_LINK>

Arguments:
  <SOURCE>        The source Architectural Decision Record number or file name match
  <LINK>          Description of the link to create in the source Architectural Decision Record
  <TARGET>        The target Architectural Decision Record number or file name match
  <REVERSE_LINK>  Description of the link to create in the target Architectural Decision Record

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

```sh
adrs link 12 Amends 10 "Amended by"
```

## Issues

See the [cmd-link](https://github.com/joshrotenberg/adrs/labels/cmd-link) label for command specific issues.
