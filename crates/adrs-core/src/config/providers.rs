//! Custom Figment providers for ADR configuration.
//!
//! This module provides custom configuration providers that integrate with
//! [figment](https://docs.rs/figment)'s layered configuration system.
//!
//! # Providers
//!
//! | Provider | Source | Priority (per ADR-0020) |
//! |----------|--------|-------------------------|
//! | [`AdrsEnv`] | `ADRS_*` env vars | Layer 1 (highest) |
//! | [`AdrsConfigEnv`] | `ADRS_CONFIG` env var | Layer 2 |
//! | [`GitConfig`] | gitconfig `[adrs]` section | Layers 3, 7, 8 |
//! | [`AdrDirFile`] | `.adr-dir` file | Layer 5 (legacy) |
//!
//! # Architecture
//!
//! These providers implement [`figment::Provider`] to participate in
//! figment's configuration merging. Each provider returns a
//! `Map<Profile, Dict>` containing its configuration values.
//!
//! When a file doesn't exist or is empty, providers return an empty map
//! (not an error), allowing higher-priority sources to provide values.
//! This follows the graceful degradation policy in ADR-0022.

use bstr::ByteSlice;
use figment::value::{Dict, Map, Value};
use figment::{Metadata, Profile, Provider};
use std::path::{Path, PathBuf};

// ============================================================================
// GitConfig Provider
// ============================================================================

/// The scope of a gitconfig file.
///
/// Determines which gitconfig file to read and affects precedence
/// per ADR-0020.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitConfigScope {
    /// Repository-local config (`.git/config`). Highest priority among git configs (Layer 3).
    Local,
    /// User global config (`~/.gitconfig`). Medium priority (Layer 7).
    Global,
    /// System-wide config (`/etc/gitconfig`). Lowest priority among git configs (Layer 8).
    System,
}

impl std::fmt::Display for GitConfigScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitConfigScope::Local => write!(f, "local"),
            GitConfigScope::Global => write!(f, "global"),
            GitConfigScope::System => write!(f, "system"),
        }
    }
}

/// Figment provider that reads `[adrs]` section from gitconfig.
///
/// Supports local (`.git/config`), global (`~/.gitconfig`), and system
/// (`/etc/gitconfig`) scopes. gix-config handles `GIT_DIR`, `GIT_CONFIG_GLOBAL`,
/// and `GIT_CONFIG_SYSTEM` environment variables automatically.
///
/// # Scopes
///
/// | Scope | Location | Env Override | Priority |
/// |-------|----------|--------------|----------|
/// | Local | `.git/config` | `GIT_DIR` | Layer 3 |
/// | Global | `~/.gitconfig` | `GIT_CONFIG_GLOBAL` | Layer 7 |
/// | System | `/etc/gitconfig` | `GIT_CONFIG_SYSTEM` | Layer 8 |
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::config::providers::GitConfig;
/// use figment::Figment;
///
/// let figment = Figment::new()
///     .merge(GitConfig::system().unwrap_or_default())
///     .merge(GitConfig::global().unwrap_or_default())
///     .merge(GitConfig::local(root).unwrap_or_default());
/// ```
///
/// # Errors
///
/// The provider itself doesn't error on construction. The `data()` method
/// handles errors per ADR-0022:
/// - Parse errors: warn and return empty map
/// - Permission denied: silent skip
/// - Missing file: return empty map
#[derive(Debug, Clone)]
pub struct GitConfig {
    scope: GitConfigScope,
    path: PathBuf,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            scope: GitConfigScope::Local,
            path: PathBuf::new(),
        }
    }
}

impl GitConfig {
    /// Create a provider for local `.git/config`.
    ///
    /// # Arguments
    ///
    /// * `root` - The repository root directory containing `.git/`
    ///
    /// # Returns
    ///
    /// Returns `Some(Self)` if a git repository exists at `root`.
    /// Returns `None` if:
    /// - `root/.git` doesn't exist
    /// - `root/.git` is a file (git worktree) - worktrees handled separately
    /// - `root/.git/config` doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::GitConfig;
    /// use tempfile::TempDir;
    /// use std::fs;
    ///
    /// let temp = TempDir::new().unwrap();
    ///
    /// // No .git directory - returns None
    /// assert!(GitConfig::local(temp.path()).is_none());
    ///
    /// // Create .git/config
    /// fs::create_dir(temp.path().join(".git")).unwrap();
    /// fs::write(temp.path().join(".git/config"), "[core]\n").unwrap();
    ///
    /// // Now returns Some
    /// assert!(GitConfig::local(temp.path()).is_some());
    /// ```
    pub fn local(root: &Path) -> Option<Self> {
        let git_dir = root.join(".git");

        // Check if .git is a directory (not a worktree file)
        if !git_dir.is_dir() {
            return None;
        }

        let config_path = git_dir.join("config");
        if !config_path.exists() {
            return None;
        }

        Some(Self {
            scope: GitConfigScope::Local,
            path: config_path,
        })
    }

    /// Create a provider for global `~/.gitconfig`.
    ///
    /// # Returns
    ///
    /// Returns `Some(Self)` if the global gitconfig exists.
    /// Returns `None` if:
    /// - Home directory cannot be determined
    /// - `~/.gitconfig` doesn't exist
    ///
    /// Respects `GIT_CONFIG_GLOBAL` environment variable.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use adrs_core::config::providers::GitConfig;
    ///
    /// if let Some(provider) = GitConfig::global() {
    ///     // Use provider in figment chain
    /// }
    /// ```
    pub fn global() -> Option<Self> {
        let path = std::env::var("GIT_CONFIG_GLOBAL")
            .map(PathBuf::from)
            .ok()
            .or_else(|| dirs::home_dir().map(|h| h.join(".gitconfig")))?;

        if !path.exists() {
            return None;
        }

        Some(Self {
            scope: GitConfigScope::Global,
            path,
        })
    }

    /// Create a provider for system `/etc/gitconfig`.
    ///
    /// # Returns
    ///
    /// Returns `Some(Self)` if the system gitconfig exists.
    /// Returns `None` if `/etc/gitconfig` doesn't exist.
    ///
    /// Respects `GIT_CONFIG_SYSTEM` environment variable.
    ///
    /// # Platform Notes
    ///
    /// - Unix: `/etc/gitconfig`
    /// - Windows: `C:\ProgramData\Git\config` (typically)
    pub fn system() -> Option<Self> {
        let path = std::env::var("GIT_CONFIG_SYSTEM")
            .map(PathBuf::from)
            .ok()
            .unwrap_or_else(|| {
                #[cfg(unix)]
                {
                    PathBuf::from("/etc/gitconfig")
                }
                #[cfg(windows)]
                {
                    PathBuf::from("C:\\ProgramData\\Git\\config")
                }
            });

        if !path.exists() {
            return None;
        }

        Some(Self {
            scope: GitConfigScope::System,
            path,
        })
    }

    /// Get the path to the gitconfig file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the scope of this gitconfig provider.
    pub fn scope(&self) -> GitConfigScope {
        self.scope
    }
}

impl Provider for GitConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named(format!("gitconfig ({})", self.scope))
            .source(self.path.display().to_string())
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        // If path is empty (default), return empty map
        if self.path.as_os_str().is_empty() {
            return Ok(Map::new());
        }

        // Read the gitconfig file using gix-config
        let file = match gix_config::File::from_path_no_includes(
            self.path.clone(),
            gix_config::Source::Local,
        ) {
            Ok(f) => f,
            Err(e) => {
                // Per ADR-0022: parse errors warn and return empty
                tracing::warn!("Failed to parse gitconfig at {:?}: {}", self.path, e);
                return Ok(Map::new());
            }
        };

        // Look for [adrs] section
        let section = match file.section("adrs", None) {
            Ok(s) => s,
            Err(_) => {
                // No [adrs] section is not an error - just return empty
                tracing::debug!("No [adrs] section in {:?}", self.path);
                return Ok(Map::new());
            }
        };

        let mut dict = Dict::new();

        // Read known keys with case-insensitive matching
        // Body implements IntoIterator yielding (ValueName, Cow<BStr>) tuples
        for (key, value) in section.body().clone() {
            if let Some(field_path) = map_gitconfig_key(key.as_ref()) {
                let value_str = value.to_str_lossy().trim().to_string();

                // Skip empty values per ADR-0022
                if value_str.is_empty() {
                    tracing::debug!("Skipping empty value for key: {}", key.as_ref());
                    continue;
                }

                // Handle nested keys (e.g., "templates.format")
                if field_path.contains('.') {
                    let parts: Vec<&str> = field_path.split('.').collect();
                    if parts.len() == 2 {
                        let nested_dict = dict
                            .entry(parts[0].into())
                            .or_insert_with(|| Value::from(Dict::new()));
                        if let Value::Dict(_, inner) = nested_dict {
                            inner.insert(parts[1].into(), Value::from(value_str));
                        }
                    }
                } else {
                    dict.insert(field_path, Value::from(value_str));
                }
            }
        }

        if dict.is_empty() {
            return Ok(Map::new());
        }

        Ok(Map::from([(Profile::Default, dict)]))
    }
}

/// Map a gitconfig key to its corresponding Config field path.
///
/// Gitconfig uses kebab-case keys; Config uses snake_case fields.
/// This function performs the translation, including handling nested
/// fields like `templates.format`.
///
/// # Arguments
///
/// * `key` - The gitconfig key (e.g., "template-format")
///
/// # Returns
///
/// Returns `Some(field_path)` for known keys.
/// Returns `None` for unknown keys (which are ignored per ADR-0022).
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(map_gitconfig_key("directory"), Some("adr_dir".into()));
/// assert_eq!(map_gitconfig_key("DIRECTORY"), Some("adr_dir".into())); // case-insensitive
/// assert_eq!(map_gitconfig_key("template-format"), Some("templates.format".into()));
/// assert_eq!(map_gitconfig_key("unknown"), None);
/// ```
fn map_gitconfig_key(key: &str) -> Option<String> {
    // Case-insensitive matching (git convention)
    match key.to_lowercase().as_str() {
        "directory" => Some("adr_dir".into()),
        "mode" => Some("mode".into()),
        "template-format" => Some("templates.format".into()),
        "template-variant" => Some("templates.variant".into()),
        "template-custom" => Some("templates.custom".into()),
        _ => {
            tracing::debug!("Ignoring unknown gitconfig key: {}", key);
            None
        }
    }
}

// ============================================================================
// AdrsEnv Provider (Environment Variables)
// ============================================================================

/// Figment provider for `ADRS_*` environment variables.
///
/// This provider has the highest priority (layer 1) per ADR-0020,
/// allowing temporary overrides for CI/scripting without modifying config files.
///
/// # Supported Variables
///
/// | Variable | Config Field | Example |
/// |----------|--------------|---------|
/// | `ADRS_DIR` | `adr_dir` | `ADRS_DIR=docs/decisions` |
/// | `ADRS_MODE` | `mode` | `ADRS_MODE=ng` |
/// | `ADRS_TEMPLATE_FORMAT` | `templates.format` | `ADRS_TEMPLATE_FORMAT=madr` |
/// | `ADRS_TEMPLATE_VARIANT` | `templates.variant` | `ADRS_TEMPLATE_VARIANT=full` |
///
/// # Deprecation
///
/// `ADR_DIRECTORY` is deprecated in favor of `ADRS_DIR`. Both work during
/// the transition period, with `ADRS_DIR` taking precedence. A warning is
/// emitted when `ADR_DIRECTORY` is used.
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::config::providers::AdrsEnv;
/// use figment::Figment;
///
/// // With ADRS_DIR=custom ADRS_MODE=ng set in environment
/// let figment = Figment::new()
///     .merge(Defaults)
///     .merge(AdrsEnv);  // Highest priority, overrides all
///
/// let config: Config = figment.extract()?;
/// assert_eq!(config.adr_dir, PathBuf::from("custom"));
/// ```
///
/// # Errors
///
/// This provider never errors. Unset variables are simply not included
/// in the returned configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdrsEnv;

impl Provider for AdrsEnv {
    /// Returns metadata describing this provider.
    fn metadata(&self) -> Metadata {
        Metadata::named("ADRS_* environment variables")
    }

    /// Reads `ADRS_*` environment variables and returns them as configuration data.
    ///
    /// # Returns
    ///
    /// Always returns `Ok`. Unset variables are simply not included.
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut dict = Dict::new();

        // ADRS_DIR → adr_dir (takes precedence over deprecated ADR_DIRECTORY)
        if let Ok(dir) = std::env::var("ADRS_DIR") {
            if !dir.trim().is_empty() {
                dict.insert("adr_dir".into(), Value::from(dir));
            }
        } else if let Ok(dir) = std::env::var("ADR_DIRECTORY") {
            // Deprecated fallback
            tracing::warn!("ADR_DIRECTORY is deprecated, use ADRS_DIR instead");
            if !dir.trim().is_empty() {
                dict.insert("adr_dir".into(), Value::from(dir));
            }
        }

        // ADRS_MODE → mode
        if let Ok(mode) = std::env::var("ADRS_MODE")
            && !mode.trim().is_empty()
        {
            dict.insert("mode".into(), Value::from(mode));
        }

        // ADRS_TEMPLATE_FORMAT → templates.format (nested)
        if let Ok(fmt) = std::env::var("ADRS_TEMPLATE_FORMAT")
            && !fmt.trim().is_empty()
        {
            let mut templates = Dict::new();
            templates.insert("format".into(), Value::from(fmt));
            dict.insert("templates".into(), Value::from(templates));
        }

        // ADRS_TEMPLATE_VARIANT → templates.variant (nested)
        if let Ok(variant) = std::env::var("ADRS_TEMPLATE_VARIANT")
            && !variant.trim().is_empty()
        {
            let templates = dict
                .entry("templates".into())
                .or_insert_with(|| Value::from(Dict::new()));
            if let Value::Dict(_, inner) = templates {
                inner.insert("variant".into(), Value::from(variant));
            }
        }

        if dict.is_empty() {
            return Ok(Map::new());
        }

        Ok(Map::from([(Profile::Default, dict)]))
    }
}

// ============================================================================
// AdrsConfigEnv Provider (Config File Path)
// ============================================================================

/// Figment provider for `ADRS_CONFIG` environment variable.
///
/// Points to a specific config file path. This is layer 2 in ADR-0020
/// precedence, below `ADRS_*` env vars but above all file-based configs.
///
/// # Behavior
///
/// Unlike optional config files (which silently skip on missing/error),
/// `ADRS_CONFIG` is **explicit**: if set but the file is not found or
/// cannot be parsed, it's an error. This follows ADR-0022's principle
/// that explicit user actions should fail loudly.
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::config::providers::AdrsConfigEnv;
/// use figment::Figment;
///
/// // With ADRS_CONFIG=/path/to/custom.toml set in environment
/// let figment = Figment::new()
///     .merge(Defaults)
///     .merge(AdrsConfigEnv);  // Loads /path/to/custom.toml
///
/// let config: Config = figment.extract()?;
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - `ADRS_CONFIG` is set but the file doesn't exist
/// - `ADRS_CONFIG` is set but the file cannot be parsed as TOML
///
/// Returns empty (no error) if `ADRS_CONFIG` is not set.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdrsConfigEnv;

impl Provider for AdrsConfigEnv {
    /// Returns metadata describing this provider.
    fn metadata(&self) -> Metadata {
        Metadata::named("ADRS_CONFIG environment variable")
    }

    /// Reads the file specified by `ADRS_CONFIG` and returns its contents.
    ///
    /// # Returns
    ///
    /// - `Ok(Map)` with config values if `ADRS_CONFIG` is set and file is valid
    /// - `Ok(Map::new())` if `ADRS_CONFIG` is not set
    /// - `Err` if `ADRS_CONFIG` is set but file doesn't exist or parse fails
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let path = match std::env::var("ADRS_CONFIG") {
            Ok(p) if !p.trim().is_empty() => PathBuf::from(p),
            _ => return Ok(Map::new()), // Not set or empty = empty map
        };

        // ADRS_CONFIG is explicit: missing file is an error
        let content = std::fs::read_to_string(&path)
            .map_err(|e| figment::Error::from(format!("ADRS_CONFIG={}: {}", path.display(), e)))?;

        // Parse as TOML
        let toml_value: toml::Value = toml::from_str(&content)
            .map_err(|e| figment::Error::from(format!("ADRS_CONFIG={}: {}", path.display(), e)))?;

        // Convert toml::Value to figment Dict
        let dict = toml_value_to_dict(toml_value)?;

        if dict.is_empty() {
            return Ok(Map::new());
        }

        Ok(Map::from([(Profile::Default, dict)]))
    }
}

/// Convert a `toml::Value` to a figment `Dict`.
///
/// This is needed because `ADRS_CONFIG` loads a TOML file that must be
/// converted to figment's internal representation.
///
/// # Arguments
///
/// * `value` - The TOML value to convert
///
/// # Returns
///
/// A figment `Dict` representing the TOML table.
///
/// # Errors
///
/// Returns an error if the root value is not a TOML table.
#[allow(clippy::result_large_err)]
fn toml_value_to_dict(value: toml::Value) -> Result<Dict, figment::Error> {
    match value {
        toml::Value::Table(table) => {
            let mut dict = Dict::new();
            for (k, v) in table {
                dict.insert(k, toml_to_figment_value(v));
            }
            Ok(dict)
        }
        _ => Err(figment::Error::from(
            "ADRS_CONFIG: expected TOML table at root",
        )),
    }
}

/// Convert a `toml::Value` to a figment `Value`.
fn toml_to_figment_value(value: toml::Value) -> Value {
    match value {
        toml::Value::String(s) => Value::from(s),
        toml::Value::Integer(i) => Value::from(i),
        toml::Value::Float(f) => Value::from(f),
        toml::Value::Boolean(b) => Value::from(b),
        toml::Value::Array(arr) => Value::from(
            arr.into_iter()
                .map(toml_to_figment_value)
                .collect::<Vec<_>>(),
        ),
        toml::Value::Table(table) => {
            let mut dict = Dict::new();
            for (k, v) in table {
                dict.insert(k, toml_to_figment_value(v));
            }
            Value::from(dict)
        }
        toml::Value::Datetime(dt) => Value::from(dt.to_string()),
    }
}

// ============================================================================
// AdrDirFile Provider (Legacy)
// ============================================================================

/// Figment provider that reads legacy `.adr-dir` files.
///
/// The `.adr-dir` file format is a single line containing the path to the
/// ADR directory. This format is compatible with the original
/// [adr-tools](https://github.com/npryce/adr-tools).
///
/// # File Format
///
/// ```text
/// doc/adr
/// ```
///
/// The file contains a single line with the relative path to the ADR directory.
/// Leading/trailing whitespace and newlines are trimmed.
///
/// # Behavior
///
/// | File State | Result |
/// |------------|--------|
/// | Exists with content | Returns `{ adr_dir: "<content>", mode: "compatible" }` |
/// | Exists but empty | Returns empty map (falls back to other sources) |
/// | Does not exist | Returns empty map (falls back to other sources) |
/// | Whitespace only | Returns empty map (falls back to other sources) |
///
/// # Mode Inference
///
/// When a `.adr-dir` file is present, it implies `ConfigMode::Compatible`
/// since this is the legacy adr-tools format. The provider automatically
/// sets `mode: "compatible"` in the returned configuration.
///
/// # Example
///
/// ```rust,ignore
/// use figment::Figment;
/// use figment::providers::Serialized;
/// use adrs_core::config::providers::AdrDirFile;
///
/// let figment = Figment::new()
///     .merge(Serialized::defaults(Config::default()))
///     .merge(AdrDirFile::new("/project/.adr-dir"));
///
/// let config: Config = figment.extract()?;
/// ```
#[derive(Debug, Clone)]
pub struct AdrDirFile {
    path: PathBuf,
}

impl AdrDirFile {
    /// Create a new provider for the given `.adr-dir` file path.
    ///
    /// The path should point to the `.adr-dir` file location. It does not
    /// need to exist - a missing file results in an empty configuration map.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use adrs_core::config::providers::AdrDirFile;
    ///
    /// let provider = AdrDirFile::new("/path/to/project/.adr-dir");
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Provider for AdrDirFile {
    /// Returns metadata describing this provider.
    ///
    /// The metadata includes:
    /// - Name: "adr-dir file"
    /// - Source: The file path being read
    fn metadata(&self) -> Metadata {
        Metadata::named("adr-dir file").source(self.path.display().to_string())
    }

    /// Reads the `.adr-dir` file and returns its contents as configuration data.
    ///
    /// # Returns
    ///
    /// - `Ok(Map)` with config values if file exists and has content
    /// - `Ok(Map::new())` (empty) if file doesn't exist or is empty
    /// - `Err` if file exists but cannot be read (permissions, etc.)
    ///
    /// # Configuration Keys
    ///
    /// When the file has content, the returned map contains:
    /// - `adr_dir`: The trimmed file contents (the ADR directory path)
    /// - `mode`: Always "compatible" (legacy format implies compatible mode)
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        // If file doesn't exist, return empty (not an error)
        if !self.path.exists() {
            return Ok(Map::new());
        }

        // Read and trim the file contents
        let contents =
            std::fs::read_to_string(&self.path).map_err(|e| figment::Error::from(e.to_string()))?;

        let adr_dir = contents.trim();

        // If empty after trimming, return empty map
        if adr_dir.is_empty() {
            return Ok(Map::new());
        }

        // Build the config dict
        let mut dict = Dict::new();
        dict.insert("adr_dir".into(), Value::from(adr_dir.to_string()));

        // Legacy .adr-dir always means Compatible mode
        dict.insert("mode".into(), Value::from("compatible"));

        let mut map = Map::new();
        map.insert(Profile::Default, dict);

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========================================================================
    // GitConfig Provider Tests
    // ========================================================================

    mod gitconfig {
        use super::*;

        #[test]
        fn local_returns_some_for_valid_git_repo() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(&config_path, "[core]\nbare = false\n").unwrap();

            let provider = GitConfig::local(temp.path());
            assert!(provider.is_some());
            assert_eq!(provider.unwrap().scope(), GitConfigScope::Local);
        }

        #[test]
        fn local_returns_none_for_non_git_dir() {
            let temp = TempDir::new().unwrap();
            let provider = GitConfig::local(temp.path());
            assert!(provider.is_none());
        }

        #[test]
        fn reads_adrs_section_from_gitconfig() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(
                &config_path,
                "[adrs]\n\tdirectory = custom/adrs\n\tmode = nextgen\n",
            )
            .unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let data = provider.data().unwrap();

            let dict = data.get(&Profile::Default).unwrap();
            assert_eq!(
                dict.get("adr_dir").and_then(|v| v.as_str()),
                Some("custom/adrs")
            );
            assert_eq!(dict.get("mode").and_then(|v| v.as_str()), Some("nextgen"));
        }

        #[test]
        fn returns_empty_when_no_adrs_section() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(&config_path, "[core]\nbare = false\n").unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let data = provider.data().unwrap();

            assert!(data.is_empty());
        }

        #[test]
        fn handles_template_format_key() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(&config_path, "[adrs]\n\ttemplate-format = nextgen\n").unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let data = provider.data().unwrap();

            let dict = data.get(&Profile::Default).unwrap();
            // template-format should map to templates.format (nested)
            if let Some(Value::Dict(_, templates)) = dict.get("templates") {
                assert_eq!(
                    templates.get("format").and_then(|v| v.as_str()),
                    Some("nextgen")
                );
            } else {
                panic!("Expected templates dict");
            }
        }

        #[test]
        fn ignores_unknown_keys() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(
                &config_path,
                "[adrs]\n\tunknown-key = value\n\tdirectory = doc/adr\n",
            )
            .unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let data = provider.data().unwrap();

            let dict = data.get(&Profile::Default).unwrap();
            // Should have adr_dir but not unknown-key
            assert!(dict.get("adr_dir").is_some());
            assert!(dict.get("unknown-key").is_none());
            assert!(dict.get("unknown_key").is_none());
        }

        #[test]
        fn skips_empty_values() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(&config_path, "[adrs]\n\tdirectory = \n\tmode = nextgen\n").unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let data = provider.data().unwrap();

            let dict = data.get(&Profile::Default).unwrap();
            // directory should be skipped (empty), mode should exist
            assert!(dict.get("adr_dir").is_none());
            assert!(dict.get("mode").is_some());
        }

        #[test]
        fn case_insensitive_key_matching() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(
                &config_path,
                "[adrs]\n\tDIRECTORY = uppercase\n\tMODE = nextgen\n",
            )
            .unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let data = provider.data().unwrap();

            let dict = data.get(&Profile::Default).unwrap();
            assert_eq!(
                dict.get("adr_dir").and_then(|v| v.as_str()),
                Some("uppercase")
            );
            assert_eq!(dict.get("mode").and_then(|v| v.as_str()), Some("nextgen"));
        }

        #[test]
        fn metadata_includes_scope() {
            let temp = TempDir::new().unwrap();
            let git_dir = temp.path().join(".git");
            std::fs::create_dir(&git_dir).unwrap();
            let config_path = git_dir.join("config");
            std::fs::write(&config_path, "[core]\nbare = false\n").unwrap();

            let provider = GitConfig::local(temp.path()).unwrap();
            let meta = provider.metadata();

            assert!(meta.name.contains("local"));
        }
    }

    // ========================================================================
    // AdrsEnv Provider Tests
    // ========================================================================

    mod adrs_env {
        use super::*;
        use std::sync::Mutex;

        // Mutex to ensure env var tests don't interfere with each other
        static ENV_MUTEX: Mutex<()> = Mutex::new(());

        fn with_env<F, T>(vars: &[(&str, Option<&str>)], f: F) -> T
        where
            F: FnOnce() -> T,
        {
            let _guard = ENV_MUTEX.lock().unwrap();

            // Save original values
            let originals: Vec<_> = vars
                .iter()
                .map(|(k, _)| (*k, std::env::var(*k).ok()))
                .collect();

            // Set new values
            // SAFETY: We hold a mutex to ensure single-threaded access to env vars
            for (key, value) in vars {
                match value {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }

            let result = f();

            // Restore original values
            // SAFETY: We hold a mutex to ensure single-threaded access to env vars
            for (key, original) in originals {
                match original {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }

            result
        }

        #[test]
        fn reads_adrs_dir_env_var() {
            with_env(
                &[("ADRS_DIR", Some("custom/dir")), ("ADR_DIRECTORY", None)],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    assert_eq!(
                        dict.get("adr_dir").and_then(|v| v.as_str()),
                        Some("custom/dir")
                    );
                },
            );
        }

        #[test]
        fn reads_adrs_mode_env_var() {
            with_env(
                &[
                    ("ADRS_MODE", Some("nextgen")),
                    ("ADRS_DIR", None),
                    ("ADR_DIRECTORY", None),
                ],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    assert_eq!(dict.get("mode").and_then(|v| v.as_str()), Some("nextgen"));
                },
            );
        }

        #[test]
        fn reads_adrs_template_format_env_var() {
            with_env(
                &[
                    ("ADRS_TEMPLATE_FORMAT", Some("madr")),
                    ("ADRS_DIR", None),
                    ("ADR_DIRECTORY", None),
                ],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    if let Some(Value::Dict(_, templates)) = dict.get("templates") {
                        assert_eq!(
                            templates.get("format").and_then(|v| v.as_str()),
                            Some("madr")
                        );
                    } else {
                        panic!("Expected templates dict");
                    }
                },
            );
        }

        #[test]
        fn adrs_dir_overrides_adr_directory() {
            with_env(
                &[
                    ("ADRS_DIR", Some("new/path")),
                    ("ADR_DIRECTORY", Some("old/path")),
                ],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    // ADRS_DIR should win
                    assert_eq!(
                        dict.get("adr_dir").and_then(|v| v.as_str()),
                        Some("new/path")
                    );
                },
            );
        }

        #[test]
        fn adr_directory_deprecated_still_works() {
            with_env(
                &[("ADRS_DIR", None), ("ADR_DIRECTORY", Some("legacy/path"))],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    assert_eq!(
                        dict.get("adr_dir").and_then(|v| v.as_str()),
                        Some("legacy/path")
                    );
                },
            );
        }

        #[test]
        fn returns_empty_when_no_vars_set() {
            with_env(
                &[
                    ("ADRS_DIR", None),
                    ("ADR_DIRECTORY", None),
                    ("ADRS_MODE", None),
                    ("ADRS_TEMPLATE_FORMAT", None),
                    ("ADRS_TEMPLATE_VARIANT", None),
                ],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    assert!(data.is_empty());
                },
            );
        }

        #[test]
        fn skips_empty_values() {
            with_env(
                &[
                    ("ADRS_DIR", Some("")),
                    ("ADR_DIRECTORY", None),
                    ("ADRS_MODE", Some("  ")),
                ],
                || {
                    let provider = AdrsEnv;
                    let data = provider.data().unwrap();

                    // Both empty, should return empty map
                    assert!(data.is_empty());
                },
            );
        }
    }

    // ========================================================================
    // AdrsConfigEnv Provider Tests
    // ========================================================================

    mod adrs_config_env {
        use super::*;
        use std::sync::Mutex;

        static ENV_MUTEX: Mutex<()> = Mutex::new(());

        fn with_env<F, T>(vars: &[(&str, Option<&str>)], f: F) -> T
        where
            F: FnOnce() -> T,
        {
            let _guard = ENV_MUTEX.lock().unwrap();

            let originals: Vec<_> = vars
                .iter()
                .map(|(k, _)| (*k, std::env::var(*k).ok()))
                .collect();

            // SAFETY: We hold a mutex to ensure single-threaded access to env vars
            for (key, value) in vars {
                match value {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }

            let result = f();

            // SAFETY: We hold a mutex to ensure single-threaded access to env vars
            for (key, original) in originals {
                match original {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }

            result
        }

        #[test]
        fn reads_config_from_adrs_config_path() {
            let temp = TempDir::new().unwrap();
            let config_path = temp.path().join("custom.toml");
            std::fs::write(
                &config_path,
                r#"
                adr_dir = "custom/adrs"
                mode = "nextgen"
                "#,
            )
            .unwrap();

            with_env(
                &[("ADRS_CONFIG", Some(config_path.to_str().unwrap()))],
                || {
                    let provider = AdrsConfigEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    assert_eq!(
                        dict.get("adr_dir").and_then(|v| v.as_str()),
                        Some("custom/adrs")
                    );
                    assert_eq!(dict.get("mode").and_then(|v| v.as_str()), Some("nextgen"));
                },
            );
        }

        #[test]
        fn returns_empty_when_adrs_config_not_set() {
            with_env(&[("ADRS_CONFIG", None)], || {
                let provider = AdrsConfigEnv;
                let data = provider.data().unwrap();

                assert!(data.is_empty());
            });
        }

        #[test]
        fn errors_when_file_not_found() {
            with_env(&[("ADRS_CONFIG", Some("/nonexistent/path.toml"))], || {
                let provider = AdrsConfigEnv;
                let result = provider.data();

                assert!(result.is_err());
                let err = result.unwrap_err().to_string();
                assert!(err.contains("ADRS_CONFIG"));
                assert!(err.contains("/nonexistent/path.toml"));
            });
        }

        #[test]
        fn errors_on_invalid_toml() {
            let temp = TempDir::new().unwrap();
            let config_path = temp.path().join("bad.toml");
            std::fs::write(&config_path, "this is not valid toml {{{}}}").unwrap();

            with_env(
                &[("ADRS_CONFIG", Some(config_path.to_str().unwrap()))],
                || {
                    let provider = AdrsConfigEnv;
                    let result = provider.data();

                    assert!(result.is_err());
                    let err = result.unwrap_err().to_string();
                    assert!(err.contains("ADRS_CONFIG"));
                },
            );
        }

        #[test]
        fn handles_nested_config() {
            let temp = TempDir::new().unwrap();
            let config_path = temp.path().join("nested.toml");
            std::fs::write(
                &config_path,
                r#"
                [templates]
                format = "madr"
                variant = "full"
                "#,
            )
            .unwrap();

            with_env(
                &[("ADRS_CONFIG", Some(config_path.to_str().unwrap()))],
                || {
                    let provider = AdrsConfigEnv;
                    let data = provider.data().unwrap();

                    let dict = data.get(&Profile::Default).unwrap();
                    if let Some(Value::Dict(_, templates)) = dict.get("templates") {
                        assert_eq!(
                            templates.get("format").and_then(|v| v.as_str()),
                            Some("madr")
                        );
                        assert_eq!(
                            templates.get("variant").and_then(|v| v.as_str()),
                            Some("full")
                        );
                    } else {
                        panic!("Expected templates dict");
                    }
                },
            );
        }
    }

    // ========================================================================
    // AdrDirFile Provider Tests
    // ========================================================================

    #[test]
    fn reads_file_contents() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".adr-dir");
        std::fs::write(&path, "custom/path").unwrap();

        let provider = AdrDirFile::new(&path);
        let data = provider.data().unwrap();

        let dict = data.get(&Profile::Default).unwrap();
        assert_eq!(
            dict.get("adr_dir").and_then(|v| v.as_str()),
            Some("custom/path")
        );
    }

    #[test]
    fn trims_whitespace_and_newlines() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".adr-dir");
        std::fs::write(&path, "  path/to/adr  \n").unwrap();

        let provider = AdrDirFile::new(&path);
        let data = provider.data().unwrap();

        let dict = data.get(&Profile::Default).unwrap();
        assert_eq!(
            dict.get("adr_dir").and_then(|v| v.as_str()),
            Some("path/to/adr")
        );
    }

    #[test]
    fn returns_empty_map_when_file_missing() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".adr-dir"); // Does not exist

        let provider = AdrDirFile::new(&path);
        let data = provider.data().unwrap();

        assert!(data.is_empty());
    }

    #[test]
    fn returns_empty_map_when_file_empty() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".adr-dir");
        std::fs::write(&path, "").unwrap();

        let provider = AdrDirFile::new(&path);
        let data = provider.data().unwrap();

        assert!(data.is_empty());
    }

    #[test]
    fn returns_empty_map_when_whitespace_only() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".adr-dir");
        std::fs::write(&path, "   \n  \t  ").unwrap();

        let provider = AdrDirFile::new(&path);
        let data = provider.data().unwrap();

        assert!(data.is_empty());
    }

    #[test]
    fn sets_compatible_mode() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".adr-dir");
        std::fs::write(&path, "doc/adr").unwrap();

        let provider = AdrDirFile::new(&path);
        let data = provider.data().unwrap();

        let dict = data.get(&Profile::Default).unwrap();
        assert_eq!(
            dict.get("mode").and_then(|v| v.as_str()),
            Some("compatible")
        );
    }

    #[test]
    fn metadata_is_descriptive() {
        let provider = AdrDirFile::new("/some/path/.adr-dir");
        let meta = provider.metadata();

        assert!(meta.name.contains("adr-dir"));
    }
}
