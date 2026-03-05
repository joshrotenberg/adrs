# Adding a Command

This guide walks through adding a new command to the `adrs` CLI.

## Step 1: Create Command Module

Create `crates/adrs-cli/src/commands/mycommand.rs`:

```rust
use adrs_core::Repository;
use clap::Args;
use anyhow::Result;

/// Description shown in help
#[derive(Args)]
pub struct MyCommand {
    /// A required argument
    pub target: String,

    /// An optional flag
    #[arg(short, long)]
    pub verbose: bool,

    /// An optional value with default
    #[arg(short, long, default_value = "default")]
    pub format: String,
}

impl MyCommand {
    pub fn run(&self) -> Result<()> {
        // Open repository
        let repo = Repository::open(".")?;

        // Use adrs-core functionality
        let adrs = repo.list()?;

        if self.verbose {
            println!("Found {} ADRs", adrs.len());
        }

        // Your implementation here

        Ok(())
    }
}
```

## Step 2: Register Module

In `crates/adrs-cli/src/commands/mod.rs`:

```rust
mod mycommand;
pub use mycommand::MyCommand;
```

## Step 3: Add to CLI

In `crates/adrs-cli/src/main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Your command description
    MyCommand(commands::MyCommand),
}
```

And in the match statement:

```rust
match &cli.command {
    // ... existing matches ...
    Commands::MyCommand(cmd) => cmd.run(),
}
```

## Step 4: Add Tests

Create `crates/adrs-cli/tests/mycommand_test.rs`:

```rust
use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_mycommand_basic() {
    let dir = tempdir().unwrap();

    // Initialize repository
    Command::cargo_bin("adrs")
        .unwrap()
        .args(["init"])
        .current_dir(&dir)
        .assert()
        .success();

    // Run your command
    Command::cargo_bin("adrs")
        .unwrap()
        .args(["mycommand", "target"])
        .current_dir(&dir)
        .assert()
        .success();
}
```

## Step 5: Add Documentation

Create `docs/src/users/commands/mycommand.md`:

```markdown
# mycommand

Description of what this command does.

## Usage

\`\`\`
adrs mycommand [OPTIONS] <TARGET>
\`\`\`

## Arguments

| Argument | Description |
|----------|-------------|
| `<TARGET>` | What to operate on |

## Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Show detailed output |
| `-f, --format <FORMAT>` | Output format (default: default) |

## Examples

\`\`\`sh
adrs mycommand foo
adrs mycommand foo --verbose
\`\`\`
```

## Checklist

- [ ] Command module created
- [ ] Module registered in `mod.rs`
- [ ] Command added to CLI enum
- [ ] Match arm added
- [ ] Tests written
- [ ] Documentation added
- [ ] SUMMARY.md updated

## See Also

- [CLI Architecture](../README.md)
- [Testing Guide](../../testing/README.md)
