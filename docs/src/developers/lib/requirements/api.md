# API Requirements

<!-- toc -->

## Repository API

### LIB-API-1: Repository Opening

```rust
// Open existing repository
Repository::open(path) -> Result<Repository>

// Open or create with defaults
Repository::open_or_default(path) -> Result<Repository>

// Initialize new repository
Repository::init(path, config) -> Result<Repository>
```

**Behavior:**
- `open` MUST fail if no configuration exists
- `open_or_default` MUST return a repository with default settings
- `init` MUST create configuration and initial ADR

### LIB-API-2: ADR Operations

```rust
// CRUD operations
repo.list() -> Result<Vec<Adr>>
repo.get(number) -> Result<Option<Adr>>
repo.create(adr) -> Result<PathBuf>
repo.update(adr) -> Result<()>

// Search
repo.find(query) -> Result<Option<Adr>>
repo.search(query) -> Result<Vec<Adr>>
```

**Behavior:**
- `list` MUST return ADRs sorted by number
- `get` MUST return `None` for non-existent ADRs (not error)
- `create` MUST auto-assign number if not provided
- `update` MUST preserve unmodified fields

### LIB-API-3: Status Operations

```rust
repo.update_status(number, status) -> Result<()>
repo.update_status_with_link(number, status, linked_to) -> Result<()>
```

**Behavior:**
- MUST update both status and file content
- MUST create bidirectional links when superseding

### LIB-API-4: Link Operations

```rust
repo.link(source, target, kind) -> Result<()>
```

**Behavior:**
- MUST create bidirectional links
- MUST fail if source or target doesn't exist

## Template API

### LIB-API-5: Template Engine

```rust
// Built-in templates
TemplateEngine::new(format, variant) -> TemplateEngine

// Custom templates
TemplateEngine::from_file(path) -> Result<TemplateEngine>

// Render
engine.render(adr) -> Result<String>
```

**Behavior:**
- MUST support all built-in formats and variants
- MUST use Jinja2/minijinja syntax
- MUST handle missing optional fields gracefully

## Configuration API

### LIB-API-6: Configuration Discovery

```rust
discover(path) -> Result<DiscoveredConfig>
```

**Behavior:**
- MUST search from path upward for configuration
- MUST prefer `adrs.toml` over `.adr-dir`
- MUST return source information (file path, type)

### LIB-API-7: Configuration Access

```rust
config.adr_dir -> PathBuf
config.mode -> ConfigMode
config.adr_path(root) -> PathBuf
```

**Behavior:**
- MUST resolve relative paths against repository root
- MUST default to `doc/adr` if not specified

## See Also

- [Type Requirements](./types.md)
- [Error Requirements](./errors.md)
