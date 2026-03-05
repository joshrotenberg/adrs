# Module Overview

<!-- toc -->

## Crate Architecture

The workspace contains two crates:

```mermaid
graph TD
    subgraph "adrs (CLI crate)"
        CLI[CLI Commands]
        MCP[MCP Server<br/>feature: mcp]
    end

    subgraph "adrs-core (Library crate)"
        Repository
        Config
        Parser
        TemplateEngine
        Linter
        Export
        Types[Types<br/>Adr, AdrStatus, AdrLink]
        Error
    end

    CLI --> Repository
    MCP --> Repository
    Repository --> Config
    Repository --> Parser
    Repository --> TemplateEngine
    Repository --> Types
    Parser --> Types
    TemplateEngine --> Types
    Linter --> Repository
    Export --> Types
```

> **Note:** MCP server functionality is a feature flag (`mcp`) within the CLI crate, not a separate crate.

## Module Dependency Graph

```mermaid
graph LR
    subgraph "adrs-core modules"
        lib[lib.rs<br/>Public API]
        repository[repository.rs]
        config[config.rs]
        parse[parse.rs]
        template[template.rs]
        types[types.rs]
        error[error.rs]
        lint[lint.rs]
        export[export.rs]
        doctor[doctor.rs]
    end

    lib --> repository
    lib --> config
    lib --> parse
    lib --> template
    lib --> types
    lib --> error
    lib --> lint
    lib --> export

    repository --> config
    repository --> parse
    repository --> template
    repository --> types
    repository --> error

    parse --> types
    parse --> error

    template --> types
    template --> error

    lint --> types
    lint --> error

    export --> types
    export --> error

    config --> error

    doctor --> repository
```

## Module Summary

| Module | Description | Public |
|--------|-------------|--------|
| `repository` | ADR CRUD operations | Yes |
| `config` | Configuration discovery | Yes |
| `types` | Core types (`Adr`, `AdrStatus`, `AdrLink`) | Yes |
| `template` | Template rendering | Yes |
| `parse` | ADR file parsing | Yes |
| `lint` | Validation and linting | Yes |
| `export` | JSON-ADR import/export | Yes |
| `error` | Error types | Yes |
| `doctor` | Legacy health checks (deprecated) | Yes |

## Module Details

### repository

The main entry point for ADR operations. Handles file I/O, configuration, and coordinates other modules.

```rust
pub struct Repository { /* ... */ }

impl Repository {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self>;
    pub fn open_or_default(root: impl Into<PathBuf>) -> Self;
    pub fn init(root: impl Into<PathBuf>, adr_dir: Option<PathBuf>, ng: bool) -> Result<Self>;
    pub fn list(&self) -> Result<Vec<Adr>>;
    pub fn get(&self, number: u32) -> Result<Option<Adr>>;
    pub fn create(&self, adr: &Adr) -> Result<PathBuf>;
    // ...
}
```

### config

Configuration discovery and management. Supports both `.adr-dir` (Compatible) and `adrs.toml` (NextGen) formats.

```rust
pub fn discover(root: impl AsRef<Path>) -> Result<DiscoveredConfig>;

pub struct Config {
    pub adr_dir: PathBuf,
    pub mode: ConfigMode,
    pub templates: TemplateConfig,
}

pub enum ConfigMode {
    Compatible,
    NextGen,
}
```

### types

Core domain types for representing ADRs, statuses, and links.

See [Core Types](./types-core.md) for detailed documentation.

### template

Template engine using minijinja (Jinja2 syntax).

```rust
pub struct TemplateEngine { /* ... */ }
pub enum TemplateFormat { Nygard, Madr }
pub enum TemplateVariant { Full, Minimal, Bare, BareMinimal }
```

### parse

ADR file parsing. Auto-detects format (Compatible vs NextGen) and extracts structured data.

```rust
pub struct Parser { /* ... */ }

impl Parser {
    pub fn parse(&self, content: &str) -> Result<Adr>;
    pub fn parse_file(&self, path: &Path) -> Result<Adr>;
}
```

### lint

Validation and linting for ADRs and repositories. Uses `mdbook-lint-rulesets` for ADR-specific rules (ADR001-ADR017).

```rust
pub fn lint_adr(adr: &Adr) -> LintReport;
pub fn lint_all(repo: &Repository) -> Result<LintReport>;
pub fn check_all(path: &Path) -> Result<LintReport>;
pub fn check_repository(repo: &Repository) -> Result<LintReport>;

pub struct LintReport {
    pub issues: Vec<Issue>,
}

pub struct Issue {
    pub severity: IssueSeverity,
    pub message: String,
    pub path: Option<PathBuf>,
    pub line: Option<usize>,
}

pub enum IssueSeverity { Info, Warning, Error }
```

### export

JSON-ADR format support for interoperability with other tools.

```rust
pub fn export_adr(adr: &Adr) -> JsonAdr;
pub fn export_repository(repo: &Repository) -> Result<JsonAdrBulkExport>;
pub fn import_to_directory(path: &Path, json: &str, options: ImportOptions) -> Result<ImportResult>;
```

## Workflows

### ADR Creation Workflow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Repository
    participant Config
    participant TemplateEngine
    participant FileSystem

    User->>CLI: adrs new "Title"
    CLI->>Repository: open(root)
    Repository->>Config: discover(root)
    Config-->>Repository: DiscoveredConfig
    CLI->>Repository: new_adr(title)
    Repository->>Repository: next_number()
    Repository->>TemplateEngine: render(format, variant, context)
    TemplateEngine-->>Repository: markdown content
    Repository->>FileSystem: write file
    FileSystem-->>Repository: path
    Repository-->>CLI: Adr
    CLI-->>User: Created 0002-title.md
```

### ADR Parsing Workflow

```mermaid
sequenceDiagram
    participant Repository
    participant Parser
    participant Types

    Repository->>Parser: parse_file(path)
    Parser->>Parser: detect format
    alt YAML Frontmatter (NextGen)
        Parser->>Parser: parse_frontmatter()
        Parser->>Parser: extract metadata
        Parser->>Parser: extract body sections
    else Legacy Format (Compatible)
        Parser->>Parser: parse_legacy()
        Parser->>Parser: extract from headings
        Parser->>Parser: parse status line
    end
    Parser->>Types: construct Adr
    Types-->>Parser: Adr
    Parser-->>Repository: Adr
```

### Configuration Loading Workflow

```mermaid
sequenceDiagram
    participant Caller
    participant Config
    participant FileSystem
    participant Environment

    Caller->>Config: discover(root)
    Config->>Environment: check ADRS_DIR
    alt ENV set
        Environment-->>Config: adr_dir path
    else ENV not set
        Config->>FileSystem: search up for .adr-dir
        alt .adr-dir found
            FileSystem-->>Config: Compatible mode
        else not found
            Config->>FileSystem: search for adrs.toml
            alt adrs.toml found
                FileSystem-->>Config: NextGen mode
            else not found
                Config->>FileSystem: check ~/.config/adrs/
                FileSystem-->>Config: Global config or defaults
            end
        end
    end
    Config-->>Caller: DiscoveredConfig
```

### Template Rendering Workflow

```mermaid
sequenceDiagram
    participant Repository
    participant TemplateEngine
    participant Minijinja

    Repository->>TemplateEngine: render(format, variant, context)
    TemplateEngine->>TemplateEngine: select template source
    alt Built-in template
        TemplateEngine->>TemplateEngine: load from TEMPLATES map
    else Custom template
        TemplateEngine->>TemplateEngine: load from file
    end
    TemplateEngine->>Minijinja: add_template(source)
    TemplateEngine->>Minijinja: render(context)
    Note over Minijinja: Variables: number, title, date,<br/>status, author, tags, etc.
    Note over Minijinja: Custom filter: pad(width)
    Minijinja-->>TemplateEngine: rendered markdown
    TemplateEngine-->>Repository: String
```

## See Also

- [Core Types](./types-core.md) - Detailed type documentation
- [ADR Alignment Matrix](./adr-alignment-matrix.md) - Implementation vs ADR compliance
- [API Documentation](https://docs.rs/adrs-core) - Full Rust API docs
