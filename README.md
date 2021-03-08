# adrs

[![crates.io](https://img.shields.io/crates/d/adrs.svg)](https://crates.io/crates/adrs)
[![docs](https://docs.rs/adrs/badge.svg)](https://docs.rs/adrs)
[![Rust](https://github.com/joshrotenberg/adrs/workflows/Rust/badge.svg)](https://github.com/joshrotenberg/adrs/actions?query=workflow%3ARust)

`adrs` is a command-line tool for managing [Architectural Decision Records][0]. 

### Status

Currently in development. The first milestone is feature parity and rough compatibility with [adr-tools][1]. 

Possible extended feature ideas:

* Support [MADR][2] in addition to Nygard.
* Integrated git support.
* Built-in HTTP server support to present ADRs via the web.


[0]: https://adr.github.io 
[1]: https://github.com/npryce/adr-tools
[2]: https://adr.github.io/madr/