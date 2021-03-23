# adrs

[![crates.io](https://img.shields.io/crates/d/adrs.svg)](https://crates.io/crates/adrs)
[![docs](https://docs.rs/adrs/badge.svg)](https://docs.rs/adrs)
[![Rust](https://github.com/joshrotenberg/adrs/workflows/Rust/badge.svg)](https://github.com/joshrotenberg/adrs/actions?query=workflow%3ARust)
[![dependency status](https://deps.rs/repo/github/joshrotenberg/adrs/status.svg)](https://deps.rs/repo/github/joshrotenberg/adrs)


`adrs` is a command-line tool for managing [Architectural Decision Records][0].

### Status

Currently in development. The first milestone is feature parity and rough compatibility with [adr-tools][1].

Possible extended feature ideas:

* Support [MADR][2] in addition to Nygard.
* Integrated git support.
* Built-in HTTP server support to present ADRs via the web.


### Installation

For now:

1. clone this repository
2. run `cargo install --path .` in the root of the repo

### Command Line

```
adrs 0.1.0

USAGE:
    adrs <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    config      Show configuration
    generate    Generate summary documentation
    help        Prints this message or the help of the given subcommand(s)
    init        Initialize a new ADR directory
    link        Link together two ADRs
    list        List the ADRs
    new         Create a new, numbered ADR
```

## Contributing
## License

## Authors

* [Josh Rotenberg][3]

[0]: https://adr.github.io
[1]: https://github.com/npryce/adr-tools
[2]: https://adr.github.io/madr/
[3]: https://github.com/joshrotenberg
