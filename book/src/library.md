# Library Usage

The `adrs-core` crate provides a Rust library for working with ADRs programmatically.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
adrs-core = "0.4"
```

## Basic Usage

### Opening a Repository

```rust
use adrs_core::Repository;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Open existing repository
    let repo = Repository::open(Path::new("."))?;
    
    // Or create with defaults if it doesn't exist
    let repo = Repository::open_or_default(Path::new("."))?;
    
    Ok(())
}
```

### Listing ADRs

```rust
use adrs_core::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(Path::new("."))?;
    
    for adr in repo.list()? {
        println!("{}: {} [{}]", adr.number, adr.title, adr.status);
    }
    
    Ok(())
}
```

### Creating an ADR

```rust
use adrs_core::{Repository, Adr, AdrStatus};

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(Path::new("."))?;
    
    let adr = Adr::new(
        repo.next_number()?,
        "Use PostgreSQL for persistence".to_string(),
    );
    
    let path = repo.create(&adr)?;
    println!("Created: {}", path.display());
    
    Ok(())
}
```

### Finding an ADR

```rust
use adrs_core::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(Path::new("."))?;
    
    // By number
    let adr = repo.get(1)?;
    
    // By search term (fuzzy)
    let adr = repo.find("postgresql")?;
    
    Ok(())
}
```

### Linking ADRs

```rust
use adrs_core::{Repository, LinkKind};

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(Path::new("."))?;
    
    repo.link(
        3,                          // source
        LinkKind::Amends,           // link type
        1,                          // target
        LinkKind::AmendedBy,        // reverse link type
    )?;
    
    Ok(())
}
```

## Configuration

### Loading Configuration

```rust
use adrs_core::{Config, discover};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Discover configuration from current directory
    let discovered = discover(Path::new("."))?;
    
    println!("Project root: {}", discovered.root.display());
    println!("ADR directory: {}", discovered.config.adr_dir.display());
    println!("Source: {:?}", discovered.source);
    
    Ok(())
}
```

### Creating Configuration

```rust
use adrs_core::Config;

let config = Config {
    adr_dir: "decisions".into(),
    mode: adrs_core::ConfigMode::NextGen,
    ..Default::default()
};
```

## Templates

### Using Built-in Templates

```rust
use adrs_core::{TemplateEngine, TemplateFormat, TemplateVariant, Adr};

fn main() -> anyhow::Result<()> {
    let engine = TemplateEngine::new(TemplateFormat::Madr, TemplateVariant::Minimal);
    
    let adr = Adr::new(1, "My Decision".to_string());
    let content = engine.render(&adr)?;
    
    println!("{}", content);
    
    Ok(())
}
```

### Custom Templates

```rust
use adrs_core::{TemplateEngine, Adr};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let engine = TemplateEngine::from_file(Path::new("templates/custom.md"))?;
    
    let adr = Adr::new(1, "My Decision".to_string());
    let content = engine.render(&adr)?;
    
    Ok(())
}
```

## Parsing ADRs

### Parse from File

```rust
use adrs_core::Parser;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let parser = Parser::new();
    let adr = parser.parse_file(Path::new("doc/adr/0001-record-decisions.md"))?;
    
    println!("Title: {}", adr.title);
    println!("Status: {}", adr.status);
    println!("Date: {:?}", adr.date);
    
    for link in &adr.links {
        println!("Link: {} -> ADR {}", link.kind, link.target);
    }
    
    Ok(())
}
```

### Parse from String

```rust
use adrs_core::Parser;

fn main() -> anyhow::Result<()> {
    let content = r#"
# 1. Use PostgreSQL

Date: 2024-01-15

## Status

Accepted

## Context

We need a database.

## Decision

Use PostgreSQL.

## Consequences

None significant.
"#;

    let parser = Parser::new();
    let adr = parser.parse(content, Some(1))?;
    
    Ok(())
}
```

## Health Checks

```rust
use adrs_core::{Repository, doctor_check};

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(Path::new("."))?;
    let report = doctor_check(&repo)?;
    
    for diagnostic in &report.diagnostics {
        println!("[{:?}] {}: {}", 
            diagnostic.severity,
            diagnostic.check,
            diagnostic.message
        );
    }
    
    if report.has_errors() {
        std::process::exit(1);
    }
    
    Ok(())
}
```

## Types

### AdrStatus

```rust
use adrs_core::AdrStatus;

let status = AdrStatus::Accepted;
let status = AdrStatus::Superseded;
let status = AdrStatus::Custom("In Review".to_string());
```

### LinkKind

```rust
use adrs_core::LinkKind;

let kind = LinkKind::Supersedes;
let kind = LinkKind::AmendedBy;
let kind = LinkKind::Custom("Depends on".to_string());
```

## Error Handling

The library uses `thiserror` for error types:

```rust
use adrs_core::{Repository, Error};

fn main() {
    match Repository::open(Path::new(".")) {
        Ok(repo) => { /* use repo */ }
        Err(Error::NotFound(path)) => {
            eprintln!("ADR repository not found at {}", path.display());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Feature Flags

The `adrs-core` crate has no optional features currently. All functionality is included by default.

## API Documentation

Full API documentation is available on [docs.rs](https://docs.rs/adrs-core).
