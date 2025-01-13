# adrs

![Crates.io Version](https://img.shields.io/crates/v/adrs)
[![crates.io](https://img.shields.io/crates/d/adrs.svg)](https://crates.io/crates/adrs)
[![Rust](https://github.com/joshrotenberg/adrs/workflows/CI/badge.svg)](https://github.com/joshrotenberg/adrs/actions?query=workflow%3ACI)
[![dependency status](https://deps.rs/repo/github/joshrotenberg/adrs/status.svg)](https://deps.rs/repo/github/joshrotenberg/adrs)

`adrs` is a command-line tool for managing [Architectural Decision Records](https://adr.github.io).

## Installation

Homebrew:

```sh
brew tap joshrotenberg/brew
brew install adrs
```

From source (requires the Rust [toolchain](https://rustup.rs)):

```sh
git clone https://github.com/joshrotenberg/adrs
cd adrs
cargo install
```

Via `cargo` (aslo requires the Rust toolchain):

```sh
cargo install adrs
```

Via a released binary:

See [Releases](https://github.com/joshrotenberg/adrs/releases).

## Command Line

```zsh
Architectural Decision Record command line tool

Usage: adrs <COMMAND>

Commands:
  init      Initializes the directory of Architecture Decision Records
  new       Create a new, numbered Architectural Decision Record
  edit      Edit an existing Architectural Decision Record
  link      Link Architectural Decision Records
  list      List Architectural Decision Records
  config    Show the current configuration
  generate  Generates summary documentation about the Architectural Decision Records
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Custom templates

The `adrs new` command allows passing in custom templates using the
[handlebars templating language](https://handlebarsjs.com/). Five template
variables are respected:

| Variable name | Value                                        |
|---------------|----------------------------------------------|
| number        | Index of ADR                                 |
| date          | Current date                                 |
| title         | Title of ADR                                 |
| superceded    | Array of markdown links to superceded ADRs   |
| linked        | Array of markdown links to linked ADRs       |

## Contributing

Contributions absolutely welcome. See the current [issues](https://github.com/joshrotenberg/adrs/issues).

## License

See [LICENSE-MIT](LICENSE-MIT) or [LICENSE-APACHE-2.0](LICENSE-APACHE-.20).
