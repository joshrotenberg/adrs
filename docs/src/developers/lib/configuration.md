# Configuration

The configuration system uses [Figment](https://docs.rs/figment) for layered configuration
with multiple providers. This enables flexible configuration from environment variables,
git config, TOML files, and more.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Config::load()                        │
│                                                          │
│  Figment::new()                                         │
│      .merge(Defaults)           // Layer 9 (lowest)    │
│      .merge(GitConfig::system)  // Layer 8             │
│      .merge(GitConfig::global)  // Layer 7             │
│      .merge(GlobalToml)         // Layer 6             │
│      .merge(AdrDirFile)         // Layer 5             │
│      .merge(ProjectToml)        // Layer 4             │
│      .merge(GitConfig::local)   // Layer 3             │
│      .merge(AdrsConfigEnv)      // Layer 2             │
│      .merge(AdrsEnv)            // Layer 1 (highest)   │
│      .extract::<Config>()                               │
└─────────────────────────────────────────────────────────┘
```

Higher layers override lower layers on a **per-key basis** (merge semantics).

## Using Configuration

```rust
use adrs_core::Config;

// Load configuration with automatic layer merging
let config = Config::load(project_root)?;

// Access configuration values
println!("ADR directory: {}", config.adr_dir.display());
println!("Mode: {:?}", config.mode);

// Check where configuration came from
println!("Source: {:?}", config.source);
```

## Writing Configuration

```rust
use adrs_core::{Config, ConfigWriteTarget, GitConfigScope};

let config = Config {
    adr_dir: "docs/decisions".into(),
    mode: ConfigMode::NextGen,
    ..Default::default()
};

// Write to adrs.toml (default for NextGen mode)
config.save(project_root)?;

// Or explicitly choose a target
config.save_to(project_root, ConfigWriteTarget::Toml)?;
config.save_to(project_root, ConfigWriteTarget::GitConfig(GitConfigScope::Local))?;
```

## Custom Providers

You can implement custom configuration providers by implementing `figment::Provider`.

### Provider Implementation Pattern

```rust
use figment::{Provider, Metadata, Profile, Error};
use figment::value::{Map, Dict};
use std::path::PathBuf;

/// Example: Read configuration from a custom source.
pub struct CustomProvider {
    path: PathBuf,
}

impl CustomProvider {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Provider for CustomProvider {
    /// Metadata describes where this configuration came from.
    /// Used by `--verbose` to show configuration sources.
    fn metadata(&self) -> Metadata {
        Metadata::named("Custom Config")
            .source(self.path.display().to_string())
    }

    /// Return configuration as a figment Dict.
    /// Keys should match Config struct field names.
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut dict = Dict::new();

        // Read your custom format here
        let content = match std::fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => return Ok(Map::new()), // Missing file = empty config
        };

        // Parse and populate dict
        // Keys must match Config field names: "adr_dir", "mode", etc.
        if let Some(dir) = parse_directory(&content) {
            dict.insert("adr_dir".into(), dir.into());
        }

        Ok(Map::from([(Profile::Default, dict)]))
    }
}
```

### Key Mapping

When your source uses different key names, map them:

```rust
fn map_key(source_key: &str) -> Option<&'static str> {
    match source_key.to_lowercase().as_str() {
        "directory" | "dir" => Some("adr_dir"),
        "mode" => Some("mode"),
        "template-format" => Some("templates.format"),  // Nested!
        _ => None,  // Unknown keys ignored
    }
}
```

### Nested Values

For nested configuration like `templates.format`:

```rust
fn set_nested(dict: &mut Dict, key: &str, value: String) {
    if key.contains('.') {
        let parts: Vec<&str> = key.split('.').collect();
        // Build nested Dict structure
        let mut nested = Dict::new();
        nested.insert(parts[1].into(), value.into());
        dict.insert(parts[0].into(), nested.into());
    } else {
        dict.insert(key.into(), value.into());
    }
}
```

### Error Handling

Follow the graceful degradation policy (see [ADR-0022](../../reference/adrs/0022-error-handling-policy.md)):

```rust
fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    match self.read_config() {
        Ok(data) => Ok(data),
        Err(e) if e.is_not_found() => {
            // Missing optional config = empty, not error
            Ok(Map::new())
        }
        Err(e) if e.is_permission_denied() => {
            // Permission denied = skip silently
            tracing::trace!("Permission denied for {:?}", self.path);
            Ok(Map::new())
        }
        Err(e) if e.is_parse_error() => {
            // Parse error = warn and skip
            tracing::warn!("Parse error in {:?}: {}", self.path, e);
            Ok(Map::new())
        }
        Err(e) => Err(e.into()),  // Unexpected errors propagate
    }
}
```

## Built-in Providers

| Provider | Source | Priority |
|----------|--------|----------|
| `AdrsEnv` | `ADRS_*` environment variables | 1 (highest) |
| `AdrsConfigEnv` | `ADRS_CONFIG` env var (file path) | 2 |
| `GitConfig::local` | `.git/config [adrs]` | 3 |
| `Toml` (project) | `adrs.toml` | 4 |
| `AdrDirFile` | `.adr-dir` (legacy) | 5 |
| `Toml` (global) | `~/.config/adrs/config.toml` | 6 |
| `GitConfig::global` | `~/.gitconfig [adrs]` | 7 |
| `GitConfig::system` | `/etc/gitconfig [adrs]` | 8 |
| `Serialized::defaults` | `Config::default()` | 9 (lowest) |

## Deprecations

### Environment Variables

| Deprecated | Replacement | Removal |
|------------|-------------|---------|
| `ADR_DIRECTORY` | `ADRS_DIR` | v0.9.0 |

The old `ADR_DIRECTORY` environment variable is deprecated. Use `ADRS_DIR` instead.
Both work during the transition period, with `ADRS_DIR` taking precedence.

### Configuration Files

| Format | Status | Notes |
|--------|--------|-------|
| `.adr-dir` | Legacy | Supported via `AdrDirFile` provider, prefer `adrs.toml` |
| `adrs.toml` | Current | Recommended for version-controlled config |
| `gitconfig [adrs]` | Current | Recommended for user-local config |

## See Also

- [ADR-0013: Figment Configuration](../../reference/adrs/0013-adopt-figment-for-configuration.md)
- [ADR-0020: Configuration Priority](../../reference/adrs/0020-config-priority.md)
- [ADR-0022: Error Handling Policy](../../reference/adrs/0022-error-handling-policy.md)
- [Figment Documentation](https://docs.rs/figment)
