# new

## Overview

`new` creates a new ADR, optionally linking it to or superceding a previous ADR. A single call can link
and/or supercede multiple previous ADRs.

## Help

```sh
Create a new, numbered Architectural Decision Record

Usage: adrs new [OPTIONS] <TITLE>...

Arguments:
  <TITLE>...  Title of the new Architectural Decision Record

Options:
  -s, --superseded <SUPERSEDED>  A reference to a previous decision to supersede with this new one
  -l, --link <LINK>              Link the new Architectural Decision to a previous Architectural Decision Record
  -T, --template <TEMPLATE>      Use a custom template when generating the new Architectural Decision Record. Relative paths are resolved with respect to the directory specified in `.adr-dir` [env: ADRS_TEMPLATE_DIR=] [default: templates/template.md]
  -h, --help                     Print help
  -V, --version                  Print version
```

## Examples

```sh
# create a new ADR
adrs new My New Decision

# create a new ADR that supercedes a previous ADR
adrs new -s 2 This is a new idea

# create a new ADR that links to a previous ADR
adrs new -l "2:Amends:Amended by" This is a better idea

# use an alternate template
# the template is resolved relative to the directory specified in `.adr-dir`
adrs new -T templates/alternate.md This is a different idea
```

## Issues

See the [cmd-new](https://github.com/joshrotenberg/adrs/labels/cmd-new) label for command specific issues.
