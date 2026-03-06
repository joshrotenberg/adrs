# Import/Export

<!-- toc -->

The export module provides JSON-ADR format support for interoperability with other tools.

## JSON-ADR Format

JSON-ADR is a standard format for exchanging ADRs between tools. `adrs` supports version 1.0.0 of the specification.

## Exporting

### Single ADR

```rust
use adrs_core::{Adr, export_adr};

let adr = Adr::new(1, "Use PostgreSQL");
let json_adr = export_adr(&adr);

// Serialize to JSON
let json = serde_json::to_string_pretty(&json_adr)?;
```

### All ADRs

```rust
use adrs_core::{Repository, export_repository};

let repo = Repository::open(".")?;
let export = export_repository(&repo)?;

// Serialize to JSON
let json = serde_json::to_string_pretty(&export)?;
```

### Directory (without repository)

```rust
use adrs_core::export_directory;

let export = export_directory("path/to/adrs")?;
```

## Importing

```rust
use adrs_core::{import_to_directory, ImportOptions};

let json = r#"{ "adrs": [...] }"#;
let options = ImportOptions::default();

let result = import_to_directory("path/to/adrs", json, options)?;

println!("Imported: {}", result.imported);
println!("Skipped: {}", result.skipped);
println!("Errors: {}", result.errors.len());
```

### Import Options

```rust
pub struct ImportOptions {
    /// Skip ADRs that already exist
    pub skip_existing: bool,

    /// Overwrite existing ADRs
    pub overwrite: bool,

    /// Use NextGen mode (YAML frontmatter)
    pub nextgen: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            skip_existing: true,
            overwrite: false,
            nextgen: false,
        }
    }
}
```

## JSON-ADR Types

### JsonAdr

```rust
pub struct JsonAdr {
    pub number: u32,
    pub title: String,
    pub date: String,
    pub status: String,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub links: Vec<JsonAdrLink>,
    pub tags: Vec<String>,
}
```

### JsonAdrLink

```rust
pub struct JsonAdrLink {
    pub target: u32,
    pub kind: String,
}
```

### JsonAdrBulkExport

```rust
pub struct JsonAdrBulkExport {
    pub schema: String,
    pub version: String,
    pub repository: RepositoryInfo,
    pub tool: ToolInfo,
    pub adrs: Vec<JsonAdr>,
}
```

## Example: Bulk Export

```rust
use adrs_core::{Repository, export_repository};
use std::fs;

let repo = Repository::open(".")?;
let export = export_repository(&repo)?;

let json = serde_json::to_string_pretty(&export)?;
fs::write("adrs-export.json", json)?;
```

## Example: Migrate from Another Tool

```rust
use adrs_core::{import_to_directory, ImportOptions};
use std::fs;

// Read export from another tool
let json = fs::read_to_string("other-tool-export.json")?;

// Import with NextGen mode
let options = ImportOptions {
    nextgen: true,
    ..Default::default()
};

let result = import_to_directory("doc/adr", &json, options)?;

println!("Successfully imported {} ADRs", result.imported);
for error in result.errors {
    eprintln!("Error: {}", error);
}
```

## See Also

- [CLI export command](../../users/commands/export.md) - User-facing export
- [CLI import command](../../users/commands/import.md) - User-facing import
- [JSON-ADR Format](https://github.com/adr/adr-log/blob/main/docs/json-adr-format.md) - Specification
