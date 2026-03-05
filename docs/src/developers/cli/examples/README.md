# CLI Examples

Code examples for extending the `adrs` CLI.

## Available Examples

| Example | Description |
|---------|-------------|
| [Adding a Command](./add-command.md) | Step-by-step guide to adding a new command |

## Quick Reference

### Command Skeleton

```rust
use adrs_core::Repository;
use clap::Args;
use anyhow::Result;

#[derive(Args)]
pub struct MyCommand {
    #[arg(short, long)]
    pub verbose: bool,
}

impl MyCommand {
    pub fn run(&self) -> Result<()> {
        let repo = Repository::open(".")?;
        // Implementation
        Ok(())
    }
}
```

### Registering Commands

In `src/commands/mod.rs`:

```rust
mod mycommand;
pub use mycommand::MyCommand;
```

In `src/main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ...
    MyCommand(commands::MyCommand),
}
```
