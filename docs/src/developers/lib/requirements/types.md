# Type Requirements

<!-- toc -->

## Core Types

### LIB-TYPE-1: Adr Structure

```rust
pub struct Adr {
    pub number: u32,
    pub title: String,
    pub date: Option<Date>,
    pub status: AdrStatus,
    pub links: Vec<AdrLink>,
    pub tags: Vec<String>,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    // MADR 4.0.0 fields
    pub decision_makers: Vec<String>,
    pub consulted: Vec<String>,
    pub informed: Vec<String>,
}
```

**Requirements:**
- `number` MUST be positive (> 0)
- `title` MUST NOT be empty
- `date` SHOULD default to today if not specified
- `status` MUST default to `Proposed`

### LIB-TYPE-2: AdrStatus

```rust
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
    Custom(String),
}
```

**Requirements:**
- MUST support standard statuses
- MUST allow custom status values
- MUST be case-insensitive when parsing
- MUST preserve original case when writing

### LIB-TYPE-3: AdrLink

```rust
pub struct AdrLink {
    pub target: u32,
    pub kind: LinkKind,
}

pub enum LinkKind {
    Supersedes,
    SupersededBy,
    Amends,
    AmendedBy,
    Related,
}
```

**Requirements:**
- `target` MUST reference a valid ADR number
- `kind` MUST have a reverse for bidirectional linking
- Links MUST be stored in both source and target ADRs

### LIB-TYPE-4: Config

```rust
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

**Requirements:**
- `adr_dir` MUST be relative to repository root
- `mode` MUST affect output format, not parsing

## Serialization

### LIB-TYPE-5: Serde Support

**Requirements:**
- All public types MUST implement `Serialize` and `Deserialize`
- Serialization MUST be compatible with JSON-ADR format
- Frontmatter MUST serialize to valid YAML

### LIB-TYPE-6: Display

**Requirements:**
- `AdrStatus` MUST implement `Display` for user-friendly output
- `Error` MUST implement `Display` with actionable messages
- Types SHOULD implement `Debug` for logging

## Equality

### LIB-TYPE-7: Comparison

**Requirements:**
- `Adr` MUST implement `PartialEq` based on number
- `AdrStatus` MUST implement `PartialEq` (case-insensitive for Custom)
- `AdrLink` MUST implement `PartialEq` on target and kind

## See Also

- [API Requirements](./api.md)
- [Core Types Documentation](../modules/types-core.md)
