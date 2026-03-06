# Library (adrs-core)

`adrs-core` is the core library powering the `adrs` CLI and MCP server. You can use it directly in your Rust projects to manage ADRs programmatically.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
adrs-core = "0.7"
```

## Quick Start

```rust
use adrs_core::{Repository, Adr, AdrStatus};

fn main() -> adrs_core::Result<()> {
    // Open an existing repository
    let repo = Repository::open(".")?;

    // List all ADRs
    for adr in repo.list()? {
        println!("{}: {} [{}]", adr.number, adr.title, adr.status);
    }

    // Create a new ADR
    let mut adr = Adr::new(2, "Use PostgreSQL for persistence");
    adr.context = "We need a database for our application.".into();
    adr.decision = "We will use PostgreSQL.".into();
    adr.consequences = "We need PostgreSQL expertise.".into();

    repo.create(&adr)?;

    Ok(())
}
```

## Documentation

- [Modules Overview](./modules/README.md) - Architecture and module structure
- [Core Types](./modules/types-core.md) - `Repository`, `Adr`, `AdrStatus`, `Config`
- [Templates](./templates.md) - Template engine usage
- [Linting](./linting.md) - Validation and linting APIs
- [Import/Export](./import-export.md) - JSON-ADR format support
- [Error Handling](./errors.md) - Error types and handling
- [Requirements](./requirements/README.md) - Design requirements

## See Also

- [API Documentation](https://docs.rs/adrs-core) - Full Rust API docs
- [ADR-0004: Library-first Architecture](../../reference/adrs/0004-library-first-architecture.md)
