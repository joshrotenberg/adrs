# CLI Development

The `adrs` CLI is a thin wrapper around `adrs-core` that handles argument parsing, user interaction, and output formatting.

## Architecture

```
adrs-cli/
├── src/
│   ├── main.rs           # Entry point, argument parsing
│   ├── commands/
│   │   ├── mod.rs        # Command dispatch
│   │   ├── init.rs       # adrs init
│   │   ├── new.rs        # adrs new
│   │   ├── edit.rs       # adrs edit
│   │   ├── list.rs       # adrs list
│   │   ├── search.rs     # adrs search
│   │   ├── status.rs     # adrs status
│   │   ├── link.rs       # adrs link
│   │   ├── export.rs     # adrs export
│   │   ├── import.rs     # adrs import
│   │   ├── generate.rs   # adrs generate
│   │   ├── template.rs   # adrs template
│   │   ├── config.rs     # adrs config
│   │   └── doctor.rs     # adrs doctor
│   └── output.rs         # Output formatting
└── Cargo.toml
```

## Design Principles

1. **Thin wrapper**: CLI code should only handle I/O and formatting
2. **No business logic**: All logic belongs in `adrs-core`
3. **Clear errors**: User-friendly error messages
4. **Consistent output**: Uniform formatting across commands

## Adding a New Command

1. Create `src/commands/mycommand.rs`
2. Add command struct with clap derive macros
3. Implement the command logic calling `adrs-core`
4. Register in `src/commands/mod.rs`
5. Add to `src/main.rs` command enum

Example:

```rust
use adrs_core::Repository;
use clap::Args;

#[derive(Args)]
pub struct MyCommand {
    /// Description of argument
    #[arg(short, long)]
    pub flag: bool,
}

impl MyCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let repo = Repository::open(".")?;
        // Use adrs-core functionality
        Ok(())
    }
}
```

## Requirements

See [CLI Requirements](./requirements/README.md) for detailed specifications.
