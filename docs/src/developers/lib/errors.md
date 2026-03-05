# Error Handling

<!-- toc -->

All fallible operations in `adrs-core` return `adrs_core::Result<T>`, which is an alias for `std::result::Result<T, adrs_core::Error>`.

## Error Type

```rust
use adrs_core::Error;

pub enum Error {
    /// Configuration or ADR not found at path
    NotFound(PathBuf),

    /// Failed to parse ADR file
    Parse {
        path: PathBuf,
        line: Option<usize>,
        message: String,
    },

    /// Configuration error
    Config(String),

    /// I/O error
    Io(std::io::Error),

    /// Template rendering error
    Template(String),

    /// Invalid ADR number
    InvalidNumber(u32),

    /// Link target not found
    LinkNotFound { source: u32, target: u32 },
}
```

## Handling Errors

### Basic Pattern

```rust
use adrs_core::{Repository, Error};

match Repository::open(".") {
    Ok(repo) => {
        // Use repository
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Matching Error Variants

```rust
use adrs_core::{Repository, Error};

match Repository::open(".") {
    Ok(repo) => println!("Opened repository"),
    Err(Error::NotFound(path)) => {
        eprintln!("No configuration found at {:?}", path);
        eprintln!("Run 'adrs init' to create one.");
    }
    Err(Error::Parse { path, line, message }) => {
        if let Some(line) = line {
            eprintln!("Parse error at {:?}:{}: {}", path, line, message);
        } else {
            eprintln!("Parse error in {:?}: {}", path, message);
        }
    }
    Err(Error::Config(msg)) => {
        eprintln!("Configuration error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

### Using the ? Operator

```rust
use adrs_core::{Repository, Adr, Result};

fn create_decision(title: &str) -> Result<()> {
    let repo = Repository::open(".")?;
    let number = repo.next_number()?;
    let adr = Adr::new(number, title);
    repo.create(&adr)?;
    Ok(())
}
```

## Error Display

All errors implement `std::fmt::Display` for user-friendly messages:

```rust
use adrs_core::Error;

let error = Error::NotFound("/path/to/project".into());
println!("{}", error);
// Output: No ADR configuration found at /path/to/project
```

## Converting Errors

### From std::io::Error

```rust
use adrs_core::Error;
use std::io;

let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
let error: Error = io_error.into();
```

### To anyhow::Error

If using `anyhow` for error handling:

```rust
use adrs_core::Repository;
use anyhow::Result;

fn main() -> Result<()> {
    let repo = Repository::open(".")?;  // Automatically converts
    Ok(())
}
```

## Best Practices

1. **Use `?` for propagation**: Let errors bubble up with context
2. **Match specific variants**: Handle expected errors gracefully
3. **Provide context**: Add information when wrapping errors
4. **Log for debugging**: Log full error details, show summaries to users

```rust
use adrs_core::{Repository, Error};
use log::error;

fn open_repo() -> adrs_core::Result<Repository> {
    Repository::open(".").map_err(|e| {
        error!("Failed to open repository: {:?}", e);
        e
    })
}
```

## See Also

- [Module Overview](./modules/README.md) - Library architecture
- [Linting](./linting.md) - Validation errors
