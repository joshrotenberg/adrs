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
3. **Clear errors**: User-friendly error messages with suggestions
4. **Consistent output**: Uniform formatting across commands

## Documentation

- [Examples](./examples/README.md) - Code examples
  - [Adding a Command](./examples/add-command.md)
- [Requirements](./requirements/README.md) - Design requirements

## Command Structure

Each command follows this pattern:

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

## See Also

- [Commands Reference](../../users/commands/README.md) - User documentation
- [Library Guide](../lib/README.md) - Using adrs-core
