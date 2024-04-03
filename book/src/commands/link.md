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
# start a new ADR directory
adr init
# create a new ADR
adrs new Do something new
# create another new ADR
adrs new Do something else
# we have three ADRs
ls doc/adr/
0001-record-architecture-decisions.md
0002-do-something-new.md
0003-do-something-else.md
# link the third to the second with an "Amends" link
adrs link 3 Amends 2 "Amended by"
```

Now the status in `0003-do-something-else.md` will be:

```markdown
## Status

Accepted

Amends [2. Do something new](0002-do-something-new.md)
```

```markdown
## Status

Accepted

Amended by [3. Do something else](0003-do-something-else.md)
```

## Issues

See the [cmd-link](https://github.com/joshrotenberg/adrs/labels/cmd-link) label for command specific issues.
