# Library Requirements

Requirements for the `adrs-core` library.

## API Requirements

### LIB-1: Repository API

```rust
// Open existing repository
Repository::open(path) -> Result<Repository>

// Open or create with defaults
Repository::open_or_default(path) -> Result<Repository>

// Initialize new repository
Repository::init(path, config) -> Result<Repository>
```

### LIB-2: ADR Operations

```rust
// CRUD operations
repo.list() -> Result<Vec<AdrSummary>>
repo.get(number) -> Result<Option<Adr>>
repo.create(adr) -> Result<PathBuf>
repo.update(adr) -> Result<()>

// Search
repo.find(query) -> Result<Option<Adr>>
repo.search(query) -> Result<Vec<Adr>>
```

### LIB-3: Template API

```rust
// Built-in templates
TemplateEngine::new(format, variant) -> TemplateEngine

// Custom templates
TemplateEngine::from_file(path) -> Result<TemplateEngine>

// Render
engine.render(adr) -> Result<String>
```

### LIB-4: Configuration API

```rust
// Discover configuration
discover(path) -> Result<Discovered>

// Access configuration
config.adr_dir -> PathBuf
config.mode -> ConfigMode
```

## Error Handling

### LIB-5: Error Types

- Library MUST define typed errors
- Library MUST NOT panic
- Library MUST NOT use `unwrap()` in public code paths
- Errors MUST be informative

```rust
pub enum Error {
    NotFound(PathBuf),
    ParseError { path: PathBuf, line: usize, message: String },
    ConfigError(String),
    // ...
}
```

## Type Requirements

### LIB-6: Core Types

```rust
pub struct Adr {
    pub number: u32,
    pub title: String,
    pub date: Option<NaiveDate>,
    pub status: AdrStatus,
    pub links: Vec<Link>,
    pub tags: Vec<String>,
    pub content: String,
}

pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
    Custom(String),
}
```

## Compatibility

### LIB-7: Parsing

- MUST parse adr-tools format
- MUST parse MADR 4.0.0 format
- MUST auto-detect format
- MUST handle malformed files gracefully

### LIB-8: Generation

- MUST generate adr-tools compatible output in Compatible mode
- MUST generate YAML frontmatter in NextGen mode
