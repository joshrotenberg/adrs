# Core Types

<!-- toc -->

## Repository

The main entry point for ADR operations.

```rust
use adrs_core::Repository;

// Open existing repository
let repo = Repository::open(".")?;

// Open or use defaults if not found
let repo = Repository::open_or_default(".");

// Initialize new repository
let repo = Repository::init(".", None, false)?;  // Compatible mode
let repo = Repository::init(".", None, true)?;   // NextGen mode
```

### Methods

| Method | Description |
|--------|-------------|
| `open(path)` | Open existing repository |
| `open_or_default(path)` | Open or use defaults |
| `init(path, adr_dir, ng)` | Initialize new repository |
| `root()` | Get repository root path |
| `config()` | Get configuration |
| `adr_path()` | Get full ADR directory path |
| `list()` | List all ADRs |
| `get(number)` | Get ADR by number |
| `find(query)` | Fuzzy search by title |
| `search(query)` | Full-text search |
| `create(&adr)` | Create new ADR |
| `update(&adr)` | Update existing ADR |
| `update_status(number, status)` | Change ADR status |
| `link(source, target, kind)` | Link two ADRs |
| `next_number()` | Get next available number |

## Adr

Represents a single Architecture Decision Record.

```rust
use adrs_core::{Adr, AdrStatus, AdrLink, LinkKind};

let mut adr = Adr::new(1, "Use Rust");
adr.status = AdrStatus::Accepted;
adr.context = "We need a systems language.".into();
adr.decision = "We will use Rust.".into();
adr.consequences = "Team needs Rust training.".into();

// MADR 4.0.0 fields
adr.decision_makers = vec!["Alice".into(), "Bob".into()];
adr.consulted = vec!["Carol".into()];
adr.informed = vec!["Dave".into()];
adr.tags = vec!["language".into(), "tooling".into()];

// Add links
adr.add_link(AdrLink {
    target: 2,
    kind: LinkKind::Supersedes,
});
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `number` | `u32` | ADR number |
| `title` | `String` | Decision title |
| `date` | `Date` | Decision date |
| `status` | `AdrStatus` | Current status |
| `links` | `Vec<AdrLink>` | Links to other ADRs |
| `decision_makers` | `Vec<String>` | Who made the decision (MADR) |
| `consulted` | `Vec<String>` | Who was consulted (MADR) |
| `informed` | `Vec<String>` | Who was informed (MADR) |
| `tags` | `Vec<String>` | Categorization tags |
| `context` | `String` | Context section content |
| `decision` | `String` | Decision section content |
| `consequences` | `String` | Consequences section content |
| `path` | `Option<PathBuf>` | File path (if loaded from file) |

### Methods

| Method | Description |
|--------|-------------|
| `new(number, title)` | Create new ADR |
| `filename()` | Get formatted filename |
| `full_title()` | Get "N. Title" format |
| `add_link(link)` | Add link to another ADR |

## AdrStatus

```rust
use adrs_core::AdrStatus;

let status = AdrStatus::Proposed;
let status = AdrStatus::Accepted;
let status = AdrStatus::Deprecated;
let status = AdrStatus::Superseded;
let status = AdrStatus::Custom("on-hold".into());
```

### Variants

| Variant | Description |
|---------|-------------|
| `Proposed` | Decision is proposed but not yet accepted |
| `Accepted` | Decision has been accepted |
| `Deprecated` | Decision is deprecated but not replaced |
| `Superseded` | Decision has been replaced by another |
| `Custom(String)` | Custom status value |

## AdrLink

```rust
use adrs_core::{AdrLink, LinkKind};

let link = AdrLink {
    target: 2,
    kind: LinkKind::Supersedes,
};
```

### LinkKind Variants

| Variant | Reverse | Description |
|---------|---------|-------------|
| `Supersedes` | `SupersededBy` | This ADR replaces another |
| `SupersededBy` | `Supersedes` | This ADR is replaced by another |
| `Amends` | `AmendedBy` | This ADR modifies another |
| `AmendedBy` | `Amends` | This ADR is modified by another |
| `Related` | `Related` | General relationship |

## Config

Configuration discovery and management.

```rust
use adrs_core::{discover, Config, ConfigMode};

// Discover configuration
let discovered = discover(".")?;
println!("ADR dir: {:?}", discovered.config.adr_dir);
println!("Mode: {:?}", discovered.config.mode);

// Check mode
match discovered.config.mode {
    ConfigMode::Compatible => println!("adr-tools compatible"),
    ConfigMode::NextGen => println!("NextGen with frontmatter"),
}
```

### ConfigMode

| Mode | Config File | Description |
|------|-------------|-------------|
| `Compatible` | `.adr-dir` | adr-tools compatible |
| `NextGen` | `adrs.toml` | YAML frontmatter support |

## See Also

- [Module Overview](./README.md) - Architecture overview
- [Error Handling](../errors.md) - Error types
- [API Documentation](https://docs.rs/adrs-core) - Full Rust API docs
