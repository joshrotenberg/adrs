//! # Configuration
//!
//! Configuration handling for ADR repositories.
//!
//! ## Overview
//!
//! This module manages ADR repository configuration, supporting both
//! legacy adr-tools compatibility and the newer `adrs.toml` format.
//!
//! | Mode | Config File | Features |
//! |------|-------------|----------|
//! | Compatible | `.adr-dir` | Basic directory, adr-tools compatible |
//! | NextGen | `adrs.toml` | Full features, YAML frontmatter |
//!
//! ## Quick Start
//!
//! ```rust
//! # use adrs_core::doctest_helpers::temp_repo;
//! let (_temp, repo) = temp_repo().unwrap();
//!
//! // Access configuration via repository
//! let config = repo.config();
//! assert!(!config.is_next_gen()); // Compatible mode by default
//! ```
//!
//! ## Configuration Discovery
//!
//! The [`discover`] function searches for configuration in order:
//! 1. Environment variable `ADRS_CONFIG`
//! 2. Search upward for `.adr-dir` or `adrs.toml`
//! 3. Global config at `~/.config/adrs/config.toml`
//! 4. Default configuration

mod providers;
pub mod xdg;

pub use providers::{AdrDirFile, AdrsConfigEnv, AdrsEnv, GitConfig, GitConfigScope};
pub use xdg::global_config_dir;

use crate::{Error, Result};
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default ADR directory name.
pub const DEFAULT_ADR_DIR: &str = "doc/adr";

/// Legacy configuration file name (adr-tools compatible).
pub const LEGACY_CONFIG_FILE: &str = ".adr-dir";

/// New configuration file name.
pub const CONFIG_FILE: &str = "adrs.toml";

/// Global configuration file name.
pub const GLOBAL_CONFIG_FILE: &str = "config.toml";

/// Environment variable for ADR directory override.
pub const ENV_ADR_DIRECTORY: &str = "ADR_DIRECTORY";

/// Environment variable for config file path override.
pub const ENV_ADRS_CONFIG: &str = "ADRS_CONFIG";

/// Configuration for an ADR repository.
///
/// # Creating Configuration
///
/// Configuration is typically loaded automatically via [`Repository::open`](crate::Repository::open),
/// but can also be created directly:
///
/// ```rust
/// use adrs_core::{Config, ConfigMode};
/// use std::path::PathBuf;
///
/// let config = Config {
///     adr_dir: PathBuf::from("docs/decisions"),
///     mode: ConfigMode::NextGen,
///     ..Default::default()
/// };
///
/// assert!(config.is_next_gen());
/// ```
///
/// # Defaults
///
/// | Field | Default |
/// |-------|---------|
/// | `adr_dir` | `doc/adr` |
/// | `mode` | `ConfigMode::Compatible` |
/// | `templates` | None |
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The directory where ADRs are stored.
    pub adr_dir: PathBuf,

    /// The mode of operation.
    pub mode: ConfigMode,

    /// Template configuration.
    #[serde(default)]
    pub templates: TemplateConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            adr_dir: PathBuf::from(DEFAULT_ADR_DIR),
            mode: ConfigMode::Compatible,
            templates: TemplateConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from the given directory.
    ///
    /// Searches for configuration files in the following precedence order
    /// (highest to lowest):
    ///
    /// 1. Environment variable `ADR_DIRECTORY` (overrides `adr_dir` only)
    /// 2. `adrs.toml` (new TOML format)
    /// 3. `.adr-dir` (legacy adr-tools format)
    /// 4. Default configuration
    ///
    /// # Errors
    ///
    /// Returns [`Error::AdrDirNotFound`] if no configuration file exists
    /// and the default `doc/adr` directory doesn't exist.
    ///
    /// Returns [`Error::ConfigError`] if the configuration file is malformed
    /// or contains invalid values (e.g., empty `adr_dir`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::Config;
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    ///
    /// // Create a legacy .adr-dir config
    /// std::fs::write(temp.path().join(".adr-dir"), "decisions").unwrap();
    ///
    /// let config = Config::load(temp.path()).unwrap();
    /// assert_eq!(config.adr_dir.to_str(), Some("decisions"));
    /// ```
    ///
    /// With `adrs.toml`:
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode};
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// std::fs::write(
    ///     temp.path().join("adrs.toml"),
    ///     r#"adr_dir = "docs/adr"
    /// mode = "ng"
    /// "#
    /// ).unwrap();
    ///
    /// let config = Config::load(temp.path()).unwrap();
    /// assert_eq!(config.mode, ConfigMode::NextGen);
    /// ```
    pub fn load(root: &Path) -> Result<Self> {
        let figment = build_figment_for_root(root);
        let config: Config = figment
            .extract()
            .map_err(|e| Error::ConfigError(e.to_string()))?;

        // Validate
        validate_config(&config)?;

        // Check if any config file actually exists
        let has_toml = root.join(CONFIG_FILE).exists();
        let has_adr_dir = root.join(LEGACY_CONFIG_FILE).exists();
        let has_default_dir = root.join(DEFAULT_ADR_DIR).exists();

        if !has_toml && !has_adr_dir && !has_default_dir {
            return Err(Error::AdrDirNotFound);
        }

        Ok(config)
    }

    /// Load configuration, or return default if not found.
    ///
    /// This is a convenience method that never fails. Use [`Config::load`]
    /// if you need to handle errors explicitly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::Config;
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// // Empty directory - no config exists
    ///
    /// let config = Config::load_or_default(temp.path());
    /// assert_eq!(config.adr_dir.to_str(), Some("doc/adr")); // Default
    /// ```
    pub fn load_or_default(root: &Path) -> Self {
        Self::load(root).unwrap_or_default()
    }

    /// Save configuration to the specified target.
    ///
    /// This is the general-purpose save method that allows explicit control
    /// over where configuration is written.
    ///
    /// # Arguments
    ///
    /// * `root` - The project root directory
    /// * `target` - Where to write the configuration
    ///
    /// # Examples
    ///
    /// Write to `adrs.toml`:
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode, ConfigWriteTarget};
    /// use tempfile::TempDir;
    /// use std::path::PathBuf;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let config = Config {
    ///     adr_dir: PathBuf::from("docs/decisions"),
    ///     mode: ConfigMode::NextGen,
    ///     ..Default::default()
    /// };
    ///
    /// config.save_to(temp.path(), ConfigWriteTarget::Toml).unwrap();
    /// assert!(temp.path().join("adrs.toml").exists());
    /// ```
    ///
    /// Write to `.adr-dir` (legacy format):
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode, ConfigWriteTarget};
    /// use tempfile::TempDir;
    /// use std::path::PathBuf;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let config = Config {
    ///     adr_dir: PathBuf::from("decisions"),
    ///     mode: ConfigMode::Compatible,
    ///     ..Default::default()
    /// };
    ///
    /// config.save_to(temp.path(), ConfigWriteTarget::LegacyAdrDir).unwrap();
    ///
    /// let content = std::fs::read_to_string(temp.path().join(".adr-dir")).unwrap();
    /// assert_eq!(content, "decisions");
    /// ```
    ///
    /// Write to local gitconfig:
    ///
    /// ```rust,ignore
    /// use adrs_core::{Config, ConfigWriteTarget, GitConfigScope};
    ///
    /// let config = Config::default();
    /// config.save_to(root, ConfigWriteTarget::GitConfig(GitConfigScope::Local))?;
    /// // Creates/updates .git/config with [adrs] section
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The target file/directory cannot be written (permissions, disk full)
    /// - For `GitConfig(Local)`: not in a git repository
    /// - For `GitConfig(Global)`: home directory cannot be determined
    /// - For `GitConfig(System)`: insufficient permissions (usually requires root)
    pub fn save_to(&self, root: &Path, target: ConfigWriteTarget) -> Result<()> {
        match target {
            ConfigWriteTarget::Toml => self.write_toml(root),
            ConfigWriteTarget::LegacyAdrDir => self.write_adr_dir(root),
            ConfigWriteTarget::GitConfig(scope) => self.write_gitconfig(root, scope),
        }
    }

    /// Save configuration to the given directory.
    ///
    /// The output format depends on the current [`ConfigMode`]:
    ///
    /// - [`ConfigMode::Compatible`]: Writes `.adr-dir` (single line with directory path)
    /// - [`ConfigMode::NextGen`]: Writes `adrs.toml` (full TOML configuration)
    ///
    /// This is a convenience method. For explicit control over the output
    /// format, use [`Config::save_to`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be written (permissions, disk full, etc.)
    /// - TOML serialization fails (should not happen with valid config)
    ///
    /// # Examples
    ///
    /// Save in Compatible mode (creates `.adr-dir`):
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode};
    /// use tempfile::TempDir;
    /// use std::path::PathBuf;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let config = Config {
    ///     adr_dir: PathBuf::from("decisions"),
    ///     mode: ConfigMode::Compatible,
    ///     ..Default::default()
    /// };
    ///
    /// config.save(temp.path()).unwrap();
    ///
    /// let content = std::fs::read_to_string(temp.path().join(".adr-dir")).unwrap();
    /// assert_eq!(content, "decisions");
    /// ```
    ///
    /// Save in NextGen mode (creates `adrs.toml`):
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode};
    /// use tempfile::TempDir;
    /// use std::path::PathBuf;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let config = Config {
    ///     adr_dir: PathBuf::from("docs/decisions"),
    ///     mode: ConfigMode::NextGen,
    ///     ..Default::default()
    /// };
    ///
    /// config.save(temp.path()).unwrap();
    ///
    /// let content = std::fs::read_to_string(temp.path().join("adrs.toml")).unwrap();
    /// assert!(content.contains("docs/decisions"));
    /// assert!(content.contains("ng")); // mode serializes as "ng"
    /// ```
    pub fn save(&self, root: &Path) -> Result<()> {
        let target = match self.mode {
            ConfigMode::Compatible => ConfigWriteTarget::LegacyAdrDir,
            ConfigMode::NextGen => ConfigWriteTarget::Toml,
        };
        self.save_to(root, target)
    }

    /// Write configuration to `adrs.toml`.
    fn write_toml(&self, root: &Path) -> Result<()> {
        let path = root.join(CONFIG_FILE);
        let content =
            toml::to_string_pretty(self).map_err(|e| Error::ConfigError(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Write configuration to `.adr-dir` (legacy format).
    fn write_adr_dir(&self, root: &Path) -> Result<()> {
        let path = root.join(LEGACY_CONFIG_FILE);
        std::fs::write(&path, self.adr_dir.display().to_string())?;
        Ok(())
    }

    /// Write configuration to gitconfig `[adrs]` section.
    fn write_gitconfig(&self, root: &Path, scope: GitConfigScope) -> Result<()> {
        use bstr::BStr;

        let path = match scope {
            GitConfigScope::Local => {
                let git_config = root.join(".git/config");
                if !git_config.exists() {
                    return Err(Error::ConfigError(
                        "Not a git repository (no .git/config)".into(),
                    ));
                }
                git_config
            }
            GitConfigScope::Global => xdg::global_gitconfig_path()?,
            GitConfigScope::System => xdg::system_gitconfig_path(),
        };

        // Read existing gitconfig or create new
        let mut file = if path.exists() {
            gix_config::File::from_path_no_includes(path.clone(), gix_config::Source::Local)
                .map_err(|e| Error::ConfigError(format!("Failed to parse gitconfig: {}", e)))?
        } else {
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            gix_config::File::new(gix_config::file::Metadata::from(gix_config::Source::Local))
        };

        // Create or get the [adrs] section
        let section_id = file
            .section_mut_or_create_new("adrs", None)
            .map_err(|e| Error::ConfigError(format!("Failed to create [adrs] section: {}", e)))?
            .id();

        // Set directory
        let dir_str = self.adr_dir.display().to_string();
        file.section_mut_by_id(section_id)
            .ok_or_else(|| Error::ConfigError("Failed to get [adrs] section".into()))?
            .set(
                "directory".try_into().unwrap(),
                BStr::new(dir_str.as_bytes()),
            );

        // Set mode
        let mode_str = match self.mode {
            ConfigMode::Compatible => "compatible",
            ConfigMode::NextGen => "ng",
        };
        file.section_mut_by_id(section_id)
            .ok_or_else(|| Error::ConfigError("Failed to get [adrs] section".into()))?
            .set("mode".try_into().unwrap(), BStr::new(mode_str.as_bytes()));

        // Write template settings if present
        if let Some(ref format) = self.templates.format {
            let format_str = format.to_string();
            file.section_mut_by_id(section_id)
                .ok_or_else(|| Error::ConfigError("Failed to get [adrs] section".into()))?
                .set(
                    "template-format".try_into().unwrap(),
                    BStr::new(format_str.as_bytes()),
                );
        }

        if let Some(ref variant) = self.templates.variant {
            file.section_mut_by_id(section_id)
                .ok_or_else(|| Error::ConfigError("Failed to get [adrs] section".into()))?
                .set(
                    "template-variant".try_into().unwrap(),
                    BStr::new(variant.as_bytes()),
                );
        }

        // Write the file
        let mut output = Vec::new();
        file.write_to(&mut output)
            .map_err(|e| Error::ConfigError(format!("Failed to serialize gitconfig: {}", e)))?;
        std::fs::write(&path, output)?;

        Ok(())
    }

    /// Returns the full path to the ADR directory.
    ///
    /// Combines the project root with the configured `adr_dir` to produce
    /// an absolute path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::Config;
    /// use std::path::{Path, PathBuf};
    ///
    /// let config = Config {
    ///     adr_dir: PathBuf::from("docs/decisions"),
    ///     ..Default::default()
    /// };
    ///
    /// let full_path = config.adr_path(Path::new("/home/user/project"));
    /// assert_eq!(full_path, PathBuf::from("/home/user/project/docs/decisions"));
    /// ```
    pub fn adr_path(&self, root: &Path) -> PathBuf {
        root.join(&self.adr_dir)
    }

    /// Returns true if running in next-gen mode.
    ///
    /// NextGen mode enables YAML frontmatter, structured links, and tags.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode};
    ///
    /// let compatible = Config::default();
    /// assert!(!compatible.is_next_gen());
    ///
    /// let nextgen = Config {
    ///     mode: ConfigMode::NextGen,
    ///     ..Default::default()
    /// };
    /// assert!(nextgen.is_next_gen());
    /// ```
    pub fn is_next_gen(&self) -> bool {
        matches!(self.mode, ConfigMode::NextGen)
    }

    /// Merge another config into this one.
    ///
    /// Values from `other` take precedence, except:
    /// - `adr_dir` is only overwritten if `other` differs from the default
    /// - `None` values in templates do not overwrite existing values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::{Config, ConfigMode};
    /// use std::path::PathBuf;
    ///
    /// let mut base = Config::default();
    ///
    /// let overlay = Config {
    ///     adr_dir: PathBuf::from("custom/path"),
    ///     mode: ConfigMode::NextGen,
    ///     ..Default::default()
    /// };
    ///
    /// base.merge(&overlay);
    ///
    /// assert_eq!(base.adr_dir, PathBuf::from("custom/path"));
    /// assert_eq!(base.mode, ConfigMode::NextGen);
    /// ```
    pub fn merge(&mut self, other: &Config) {
        // adr_dir: use other if it differs from default
        if other.adr_dir.as_os_str() != DEFAULT_ADR_DIR {
            self.adr_dir = other.adr_dir.clone();
        }
        // mode: other takes precedence
        self.mode = other.mode;
        // templates: merge
        if other.templates.format.is_some() {
            self.templates.format = other.templates.format.clone();
        }
        if other.templates.variant.is_some() {
            self.templates.variant = other.templates.variant.clone();
        }
        if other.templates.custom.is_some() {
            self.templates.custom = other.templates.custom.clone();
        }
    }
}

/// Result of discovering configuration.
///
/// Returned by [`discover`] with the resolved configuration,
/// the project root directory, and the source of the configuration.
///
/// # Examples
///
/// ```rust
/// # use adrs_core::doctest_helpers::temp_repo;
/// use adrs_core::{discover, ConfigSource, GitConfigScope};
///
/// let (temp, _repo) = temp_repo().unwrap();
///
/// let discovered = discover(temp.path()).unwrap();
///
/// // Access the resolved configuration
/// println!("ADR directory: {}", discovered.config.adr_dir.display());
///
/// // Check where it was loaded from
/// match discovered.source {
///     ConfigSource::Project(path) => println!("From project: {}", path.display()),
///     ConfigSource::LegacyProject(path) => println!("From legacy: {}", path.display()),
///     ConfigSource::Global(path) => println!("From global: {}", path.display()),
///     ConfigSource::GitConfig(scope) => println!("From gitconfig: {:?}", scope),
///     ConfigSource::Environment => println!("From environment variables"),
///     ConfigSource::EnvironmentConfig(path) => println!("From ADRS_CONFIG: {}", path.display()),
///     ConfigSource::Default => println!("Using defaults"),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DiscoveredConfig {
    /// The resolved configuration.
    pub config: Config,
    /// The project root directory (where config was found).
    pub root: PathBuf,
    /// Where the config was loaded from.
    pub source: ConfigSource,
}

/// Indicates where the configuration was loaded from.
///
/// # Examples
///
/// ```rust
/// use adrs_core::ConfigSource;
/// use std::path::PathBuf;
///
/// let source = ConfigSource::Project(PathBuf::from("adrs.toml"));
/// assert!(matches!(source, ConfigSource::Project(_)));
///
/// let default = ConfigSource::Default;
/// assert_eq!(default, ConfigSource::Default);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// Loaded from ADRS_* environment variables (Layer 1).
    ///
    /// This is the highest priority source. ADRS_DIR, ADRS_MODE, etc.
    Environment,

    /// Loaded from ADRS_CONFIG environment variable (Layer 2).
    ///
    /// Points to a specific config file path.
    EnvironmentConfig(PathBuf),

    /// Loaded from gitconfig `[adrs]` section (Layers 3, 7, 8).
    ///
    /// The scope indicates local (.git/config), global (~/.gitconfig),
    /// or system (/etc/gitconfig).
    GitConfig(GitConfigScope),

    /// Loaded from project config file `adrs.toml` (Layer 4).
    Project(PathBuf),

    /// Loaded from legacy `.adr-dir` file (Layer 5).
    ///
    /// This is the adr-tools compatible format.
    LegacyProject(PathBuf),

    /// Loaded from user global config (Layer 6).
    ///
    /// Located at `~/.config/adrs/config.toml` or XDG equivalent.
    Global(PathBuf),

    /// Using defaults (Layer 9).
    ///
    /// No config file found; using built-in defaults.
    Default,
}

/// Target destination for writing configuration.
///
/// Used by [`Config::save_to`] to specify where configuration should be saved.
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::{Config, ConfigWriteTarget, GitConfigScope};
///
/// let config = Config::default();
///
/// // Write to adrs.toml
/// config.save_to(root, ConfigWriteTarget::Toml)?;
///
/// // Write to .adr-dir (legacy)
/// config.save_to(root, ConfigWriteTarget::LegacyAdrDir)?;
///
/// // Write to local gitconfig
/// config.save_to(root, ConfigWriteTarget::GitConfig(GitConfigScope::Local))?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigWriteTarget {
    /// Write to `adrs.toml` (NextGen mode default).
    ///
    /// Version-controlled, TOML format with all fields.
    Toml,

    /// Write to `.adr-dir` (Compatible mode default).
    ///
    /// Version-controlled, plain text with directory path only.
    LegacyAdrDir,

    /// Write to gitconfig `[adrs]` section.
    ///
    /// Not version-controlled (unless using global/system scope).
    GitConfig(GitConfigScope),
}

/// Discovers configuration by searching up the directory tree.
///
/// Search order:
/// 1. Environment variable `ADRS_CONFIG` (explicit config path)
/// 2. Search upward from `start_dir` for `.adr-dir` or `adrs.toml`
/// 3. Global config at `~/.config/adrs/config.toml`
/// 4. Default configuration
///
/// Environment variable `ADR_DIRECTORY` overrides the ADR directory.
///
/// # Examples
///
/// ```rust
/// # use adrs_core::doctest_helpers::temp_repo;
/// use adrs_core::{discover, ConfigSource};
///
/// let (temp, _repo) = temp_repo().unwrap();
///
/// let discovered = discover(temp.path()).unwrap();
/// assert_eq!(discovered.root, temp.path());
/// assert!(matches!(discovered.source, ConfigSource::Project(_)));
/// ```
pub fn discover(start_dir: &Path) -> Result<DiscoveredConfig> {
    // Check for explicit config path from environment
    if let Ok(config_path) = std::env::var(ENV_ADRS_CONFIG) {
        let path = PathBuf::from(&config_path);
        if path.exists() {
            let figment = build_figment_for_explicit_config(&path);
            let config: Config = figment
                .extract()
                .map_err(|e| Error::ConfigError(e.to_string()))?;
            return Ok(DiscoveredConfig {
                config,
                root: path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| start_dir.to_path_buf()),
                source: ConfigSource::Environment,
            });
        }
    }

    // Search upward for project config
    if let Some((root, source)) = find_project_root(start_dir) {
        let figment = build_figment_for_root(&root);
        let config: Config = figment
            .extract()
            .map_err(|e| Error::ConfigError(e.to_string()))?;
        return Ok(DiscoveredConfig {
            config,
            root,
            source,
        });
    }

    // Try global config
    if let Some(global_path) = find_global_config() {
        let figment = build_figment_for_global(&global_path);
        let config: Config = figment
            .extract()
            .map_err(|e| Error::ConfigError(e.to_string()))?;
        return Ok(DiscoveredConfig {
            config,
            root: start_dir.to_path_buf(),
            source: ConfigSource::Global(global_path),
        });
    }

    // Use defaults with env overrides
    let figment = Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(
            Env::raw()
                .only(&[ENV_ADR_DIRECTORY])
                .map(|_| "adr_dir".into()),
        );

    let config: Config = figment
        .extract()
        .map_err(|e| Error::ConfigError(e.to_string()))?;

    Ok(DiscoveredConfig {
        config,
        root: start_dir.to_path_buf(),
        source: ConfigSource::Default,
    })
}

/// Find the project root by searching upward.
///
/// Returns the root directory and the config source if found.
fn find_project_root(start_dir: &Path) -> Option<(PathBuf, ConfigSource)> {
    let mut current = start_dir.to_path_buf();

    loop {
        // Check for adrs.toml first (higher priority)
        let config_path = current.join(CONFIG_FILE);
        if config_path.exists() {
            return Some((current, ConfigSource::Project(config_path)));
        }

        // Check for .adr-dir
        let legacy_path = current.join(LEGACY_CONFIG_FILE);
        if legacy_path.exists() {
            return Some((current, ConfigSource::Project(legacy_path)));
        }

        // Check for default ADR directory (indicates project root)
        let default_dir = current.join(DEFAULT_ADR_DIR);
        if default_dir.exists() {
            return Some((current, ConfigSource::Default));
        }

        // Stop at git repository root
        if current.join(".git").exists() {
            break;
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }

    None
}

/// Find the global config file path if it exists.
fn find_global_config() -> Option<PathBuf> {
    let config_dir = dirs_config_dir().ok()?;
    let global_path = config_dir.join("adrs").join(GLOBAL_CONFIG_FILE);

    if global_path.exists() {
        Some(global_path)
    } else {
        None
    }
}

/// Build a figment for a known project root.
/// Build a Figment with full 9-layer precedence per ADR-0020.
///
/// Layers are merged per-key (higher layers override lower layers):
///
/// | Layer | Source | Priority |
/// |-------|--------|----------|
/// | 9 | Built-in defaults | Lowest |
/// | 8 | System gitconfig (`/etc/gitconfig`) | |
/// | 7 | Global gitconfig (`~/.gitconfig`) | |
/// | 6 | User global config (`~/.config/adrs/config.toml`) | |
/// | 5 | Legacy `.adr-dir` file | |
/// | 4 | Project `adrs.toml` | |
/// | 3 | Local gitconfig (`.git/config`) | |
/// | 2 | `ADRS_CONFIG` env var (explicit file path) | |
/// | 1 | `ADRS_*` env vars | Highest |
fn build_figment_for_root(root: &Path) -> Figment {
    // Start with defaults (Layer 9)
    let mut figment = Figment::new().merge(Serialized::defaults(Config::default()));

    // Layer 8: System gitconfig
    if let Some(system_git) = GitConfig::system() {
        figment = figment.merge(system_git);
    }

    // Layer 7: Global gitconfig
    if let Some(global_git) = GitConfig::global() {
        figment = figment.merge(global_git);
    }

    // Layer 6: User global config (~/.config/adrs/config.toml)
    if let Some(global_dir) = xdg::global_config_dir() {
        let global_config_file = global_dir.join("config.toml");
        if global_config_file.exists() {
            figment = figment.merge(Toml::file(global_config_file));
        }
    }

    // Layer 5: Legacy .adr-dir
    figment = figment.merge(AdrDirFile::new(root.join(LEGACY_CONFIG_FILE)));

    // Layer 4: Project adrs.toml
    figment = figment.merge(Toml::file(root.join(CONFIG_FILE)));

    // Layer 3: Local gitconfig (.git/config)
    if let Some(local_git) = GitConfig::local(root) {
        figment = figment.merge(local_git);
    }

    // Layer 2: ADRS_CONFIG env var (explicit file path)
    figment = figment.merge(AdrsConfigEnv);

    // Layer 1 (highest): ADRS_* env vars
    figment = figment.merge(AdrsEnv);

    figment
}

/// Build a figment for an explicit config file path (from ADRS_CONFIG env var).
fn build_figment_for_explicit_config(config_path: &Path) -> Figment {
    Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(config_path))
        .merge(
            Env::raw()
                .only(&[ENV_ADR_DIRECTORY])
                .map(|_| "adr_dir".into()),
        )
}

/// Build a figment for global config.
fn build_figment_for_global(global_path: &Path) -> Figment {
    Figment::new()
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file(global_path))
        .merge(
            Env::raw()
                .only(&[ENV_ADR_DIRECTORY])
                .map(|_| "adr_dir".into()),
        )
}

/// Validate a loaded config.
fn validate_config(config: &Config) -> Result<()> {
    if config.adr_dir.as_os_str().is_empty() {
        return Err(Error::ConfigError("adr_dir cannot be empty".into()));
    }
    Ok(())
}

/// Get the user's config directory.
fn dirs_config_dir() -> Result<PathBuf> {
    // Try XDG_CONFIG_HOME first, then fall back to ~/.config
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config"));
    }

    // Windows fallback
    if let Ok(appdata) = std::env::var("APPDATA") {
        return Ok(PathBuf::from(appdata));
    }

    Err(Error::ConfigError(
        "Could not determine config directory".into(),
    ))
}

/// The mode of operation for the ADR tool.
///
/// # Comparison
///
/// | Feature | Compatible | NextGen |
/// |---------|------------|---------|
/// | Config file | `.adr-dir` | `adrs.toml` |
/// | Frontmatter | No | YAML |
/// | Structured links | No | Yes |
/// | Tags | No | Yes |
///
/// # Examples
///
/// ```rust
/// use adrs_core::ConfigMode;
///
/// let mode: ConfigMode = Default::default();
/// assert_eq!(mode, ConfigMode::Compatible);
///
/// // Parse from string (for TOML deserialization)
/// assert_eq!(ConfigMode::Compatible, ConfigMode::Compatible);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigMode {
    /// Compatible with adr-tools (markdown-only, no frontmatter).
    #[default]
    Compatible,

    /// Next-gen mode with YAML frontmatter and enhanced features.
    #[serde(rename = "ng", alias = "nextgen")]
    NextGen,
}

/// Template configuration for ADR creation.
///
/// Controls the default template format, variant, and custom template path
/// used when creating new ADRs.
///
/// # Supported Formats
///
/// | Format | Description |
/// |--------|-------------|
/// | `nygard` | Classic Michael Nygard format (default) |
/// | `madr` | MADR 4.0.0 format with structured sections |
///
/// # Supported Variants
///
/// | Variant | Description |
/// |---------|-------------|
/// | `full` | Complete template with all sections (default) |
/// | `minimal` | Condensed template for quick decisions |
/// | `bare` | Just the essential structure |
///
/// # Examples
///
/// In `adrs.toml`:
///
/// ```toml
/// [templates]
/// format = "madr"
/// variant = "minimal"
/// # custom = "templates/my-adr.md"  # Optional custom template
/// ```
///
/// Programmatically:
///
/// ```rust
/// use adrs_core::Config;
///
/// let config = Config::default();
///
/// // Templates are None by default (uses built-in defaults)
/// assert!(config.templates.format.is_none());
/// assert!(config.templates.variant.is_none());
/// assert!(config.templates.custom.is_none());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateConfig {
    /// The default template format to use (e.g., "nygard", "madr").
    ///
    /// When `None`, defaults to "nygard".
    pub format: Option<String>,

    /// The default template variant to use (e.g., "full", "minimal", "bare").
    ///
    /// When `None`, defaults to "full".
    pub variant: Option<String>,

    /// Path to a custom template file.
    ///
    /// When set, this template is used instead of built-in templates.
    /// The path is relative to the project root.
    pub custom: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use test_case::test_case;

    // ========== Default and Constants Tests ==========

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
        assert_eq!(config.mode, ConfigMode::Compatible);
        assert!(config.templates.format.is_none());
        assert!(config.templates.custom.is_none());
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_ADR_DIR, "doc/adr");
        assert_eq!(LEGACY_CONFIG_FILE, ".adr-dir");
        assert_eq!(CONFIG_FILE, "adrs.toml");
    }

    #[test]
    fn test_config_mode_default() {
        assert_eq!(ConfigMode::default(), ConfigMode::Compatible);
    }

    // ========== Load Configuration Tests ==========

    #[test]
    fn test_load_legacy_config() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "decisions").unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("decisions"));
        assert_eq!(config.mode, ConfigMode::Compatible);
    }

    #[test]
    fn test_load_legacy_config_with_whitespace() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "  decisions  \n").unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("decisions"));
    }

    #[test]
    fn test_load_legacy_config_nested_path() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "docs/architecture/decisions").unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("docs/architecture/decisions"));
    }

    #[test]
    fn test_load_new_config() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "docs/decisions"
mode = "ng"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("docs/decisions"));
        assert_eq!(config.mode, ConfigMode::NextGen);
    }

    #[test]
    fn test_load_new_config_compatible_mode() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "doc/adr"
mode = "compatible"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.mode, ConfigMode::Compatible);
    }

    #[test]
    fn test_load_new_config_with_templates() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "decisions"
mode = "ng"

[templates]
format = "markdown"
custom = "templates/adr.md"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.templates.format, Some("markdown".to_string()));
        assert_eq!(
            config.templates.custom,
            Some(PathBuf::from("templates/adr.md"))
        );
    }

    #[test]
    fn test_load_new_config_with_template_variant() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "decisions"
mode = "ng"

[templates]
format = "madr"
variant = "minimal"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.templates.format, Some("madr".to_string()));
        assert_eq!(config.templates.variant, Some("minimal".to_string()));
    }

    #[test]
    fn test_load_new_config_with_nextgen_alias() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "decisions"
mode = "nextgen"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.mode, ConfigMode::NextGen);
    }

    #[test]
    fn test_load_new_config_minimal() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = "adrs""#).unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("adrs"));
        // Should use defaults for missing fields
        assert_eq!(config.mode, ConfigMode::Compatible);
    }

    #[test]
    fn test_load_prefers_new_config_over_legacy() {
        let temp = TempDir::new().unwrap();
        // Create both config files
        std::fs::write(temp.path().join(".adr-dir"), "legacy-dir").unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = "new-dir""#).unwrap();

        let config = Config::load(temp.path()).unwrap();
        // Should prefer adrs.toml
        assert_eq!(config.adr_dir, PathBuf::from("new-dir"));
    }

    #[test]
    fn test_load_default_dir_exists() {
        let temp = TempDir::new().unwrap();
        // Create the default directory
        std::fs::create_dir_all(temp.path().join("doc/adr")).unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_load_no_config_no_default_dir() {
        let temp = TempDir::new().unwrap();
        // Empty directory - no config, no default dir

        let result = Config::load(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_or_default_returns_default_on_error() {
        let temp = TempDir::new().unwrap();
        // Empty directory - would error with load()

        let config = Config::load_or_default(temp.path());
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
        assert_eq!(config.mode, ConfigMode::Compatible);
    }

    #[test]
    fn test_load_or_default_returns_config_when_exists() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "custom-dir").unwrap();

        let config = Config::load_or_default(temp.path());
        assert_eq!(config.adr_dir, PathBuf::from("custom-dir"));
    }

    // ========== Save Configuration Tests ==========

    #[test]
    fn test_save_legacy_config() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            adr_dir: PathBuf::from("my/adrs"),
            mode: ConfigMode::Compatible,
            templates: TemplateConfig::default(),
        };

        config.save(temp.path()).unwrap();

        let content = std::fs::read_to_string(temp.path().join(".adr-dir")).unwrap();
        assert_eq!(content, "my/adrs");
        // Should not create adrs.toml
        assert!(!temp.path().join("adrs.toml").exists());
    }

    #[test]
    fn test_save_new_config() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            adr_dir: PathBuf::from("docs/decisions"),
            mode: ConfigMode::NextGen,
            templates: TemplateConfig::default(),
        };

        config.save(temp.path()).unwrap();

        let content = std::fs::read_to_string(temp.path().join("adrs.toml")).unwrap();
        assert!(content.contains("docs/decisions"));
        assert!(content.contains("ng"));
        // Should not create .adr-dir
        assert!(!temp.path().join(".adr-dir").exists());
    }

    #[test]
    fn test_save_new_config_with_templates() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            adr_dir: PathBuf::from("decisions"),
            mode: ConfigMode::NextGen,
            templates: TemplateConfig {
                format: Some("custom".to_string()),
                variant: None,
                custom: Some(PathBuf::from("my-template.md")),
            },
        };

        config.save(temp.path()).unwrap();

        let content = std::fs::read_to_string(temp.path().join("adrs.toml")).unwrap();
        assert!(content.contains("custom"));
        assert!(content.contains("my-template.md"));
    }

    #[test]
    fn test_save_and_load_roundtrip_compatible() {
        let temp = TempDir::new().unwrap();
        let original = Config {
            adr_dir: PathBuf::from("architecture/decisions"),
            mode: ConfigMode::Compatible,
            templates: TemplateConfig::default(),
        };

        original.save(temp.path()).unwrap();
        let loaded = Config::load(temp.path()).unwrap();

        assert_eq!(loaded.adr_dir, original.adr_dir);
        assert_eq!(loaded.mode, ConfigMode::Compatible);
    }

    #[test]
    fn test_save_and_load_roundtrip_nextgen() {
        let temp = TempDir::new().unwrap();
        let original = Config {
            adr_dir: PathBuf::from("docs/adr"),
            mode: ConfigMode::NextGen,
            templates: TemplateConfig {
                format: Some("markdown".to_string()),
                variant: None,
                custom: None,
            },
        };

        original.save(temp.path()).unwrap();
        let loaded = Config::load(temp.path()).unwrap();

        assert_eq!(loaded.adr_dir, original.adr_dir);
        assert_eq!(loaded.mode, ConfigMode::NextGen);
        assert_eq!(loaded.templates.format, Some("markdown".to_string()));
    }

    // ========== save_to Tests ==========

    #[test]
    fn test_save_to_toml() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            adr_dir: PathBuf::from("my/adrs"),
            mode: ConfigMode::Compatible, // Mode doesn't matter for save_to
            templates: TemplateConfig::default(),
        };

        config
            .save_to(temp.path(), ConfigWriteTarget::Toml)
            .unwrap();

        assert!(temp.path().join("adrs.toml").exists());
        let content = std::fs::read_to_string(temp.path().join("adrs.toml")).unwrap();
        assert!(content.contains("my/adrs"));
    }

    #[test]
    fn test_save_to_legacy_adr_dir() {
        let temp = TempDir::new().unwrap();
        let config = Config {
            adr_dir: PathBuf::from("docs/decisions"),
            mode: ConfigMode::NextGen, // Mode doesn't matter for save_to
            templates: TemplateConfig::default(),
        };

        config
            .save_to(temp.path(), ConfigWriteTarget::LegacyAdrDir)
            .unwrap();

        assert!(temp.path().join(".adr-dir").exists());
        let content = std::fs::read_to_string(temp.path().join(".adr-dir")).unwrap();
        assert_eq!(content, "docs/decisions");
    }

    #[test]
    fn test_save_to_local_gitconfig() {
        let temp = TempDir::new().unwrap();
        // Create .git/config so we're "in a repo"
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        std::fs::write(temp.path().join(".git/config"), "[core]\nbare = false\n").unwrap();

        let config = Config {
            adr_dir: PathBuf::from("docs/adr"),
            mode: ConfigMode::NextGen,
            templates: TemplateConfig::default(),
        };

        config
            .save_to(
                temp.path(),
                ConfigWriteTarget::GitConfig(GitConfigScope::Local),
            )
            .unwrap();

        let content = std::fs::read_to_string(temp.path().join(".git/config")).unwrap();
        assert!(content.contains("[adrs]"));
        assert!(content.contains("directory = docs/adr"));
        assert!(content.contains("mode = ng"));
    }

    #[test]
    fn test_save_to_gitconfig_with_templates() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        std::fs::write(temp.path().join(".git/config"), "[core]\nbare = false\n").unwrap();

        let config = Config {
            adr_dir: PathBuf::from("decisions"),
            mode: ConfigMode::Compatible,
            templates: TemplateConfig {
                format: Some("madr".to_string()),
                variant: Some("minimal".to_string()),
                custom: None,
            },
        };

        config
            .save_to(
                temp.path(),
                ConfigWriteTarget::GitConfig(GitConfigScope::Local),
            )
            .unwrap();

        let content = std::fs::read_to_string(temp.path().join(".git/config")).unwrap();
        assert!(content.contains("[adrs]"));
        assert!(content.contains("template-format = madr"));
        assert!(content.contains("template-variant = minimal"));
    }

    #[test]
    fn test_save_to_gitconfig_fails_without_git_repo() {
        let temp = TempDir::new().unwrap();
        // No .git directory

        let config = Config::default();
        let result = config.save_to(
            temp.path(),
            ConfigWriteTarget::GitConfig(GitConfigScope::Local),
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Not a git repository"));
    }

    // ========== Helper Method Tests ==========

    #[test_case("doc/adr", "/project" => PathBuf::from("/project/doc/adr"); "default path")]
    #[test_case("decisions", "/home/user/repo" => PathBuf::from("/home/user/repo/decisions"); "simple path")]
    #[test_case("docs/architecture/decisions", "/repo" => PathBuf::from("/repo/docs/architecture/decisions"); "nested path")]
    fn test_adr_path(adr_dir: &str, root: &str) -> PathBuf {
        let config = Config {
            adr_dir: PathBuf::from(adr_dir),
            ..Default::default()
        };
        config.adr_path(Path::new(root))
    }

    #[test]
    fn test_is_next_gen() {
        let compatible = Config {
            mode: ConfigMode::Compatible,
            ..Default::default()
        };
        assert!(!compatible.is_next_gen());

        let nextgen = Config {
            mode: ConfigMode::NextGen,
            ..Default::default()
        };
        assert!(nextgen.is_next_gen());
    }

    // ========== ConfigMode Tests ==========

    #[test]
    fn test_config_mode_equality() {
        assert_eq!(ConfigMode::Compatible, ConfigMode::Compatible);
        assert_eq!(ConfigMode::NextGen, ConfigMode::NextGen);
        assert_ne!(ConfigMode::Compatible, ConfigMode::NextGen);
    }

    #[test]
    fn test_config_mode_serialization_in_config() {
        // TOML requires enums to be serialized within a struct
        let config = Config {
            mode: ConfigMode::Compatible,
            ..Default::default()
        };
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("mode = \"compatible\""));

        let config = Config {
            mode: ConfigMode::NextGen,
            ..Default::default()
        };
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("mode = \"ng\""));
    }

    #[test]
    fn test_config_mode_deserialization_in_config() {
        let config: Config = toml::from_str(r#"mode = "compatible""#).unwrap();
        assert_eq!(config.mode, ConfigMode::Compatible);

        let config: Config = toml::from_str(r#"mode = "ng""#).unwrap();
        assert_eq!(config.mode, ConfigMode::NextGen);
    }

    #[test]
    fn test_config_mode_deserialization_nextgen_alias() {
        let config: Config = toml::from_str(r#"mode = "nextgen""#).unwrap();
        assert_eq!(config.mode, ConfigMode::NextGen);
    }

    // ========== TemplateConfig Tests ==========

    #[test]
    fn test_template_config_default() {
        let config = TemplateConfig::default();
        assert!(config.format.is_none());
        assert!(config.variant.is_none());
        assert!(config.custom.is_none());
    }

    #[test]
    fn test_template_config_serialization() {
        let config = TemplateConfig {
            format: Some("nygard".to_string()),
            variant: None,
            custom: Some(PathBuf::from("templates/custom.md")),
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("nygard"));
        assert!(toml.contains("templates/custom.md"));
    }

    // ========== Error Cases ==========

    #[test]
    fn test_load_invalid_toml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("adrs.toml"), "this is not valid toml {{{").unwrap();

        let result = Config::load(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_empty_toml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("adrs.toml"), "").unwrap();

        // Empty TOML should use defaults
        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_load_empty_adr_dir_file() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "").unwrap();

        // Empty .adr-dir falls back to defaults (figment behavior)
        // then we check if any config exists - .adr-dir exists, so it should work
        // but the adr_dir will be default
        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
    }

    // ========== Config Discovery Tests ==========

    #[test]
    fn test_discover_finds_config_in_current_dir() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "decisions").unwrap();

        let discovered = discover(temp.path()).unwrap();
        assert_eq!(discovered.root, temp.path());
        assert_eq!(discovered.config.adr_dir, PathBuf::from("decisions"));
        assert!(matches!(discovered.source, ConfigSource::Project(_)));
    }

    #[test]
    fn test_discover_finds_config_in_parent_dir() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("src").join("lib");
        std::fs::create_dir_all(&subdir).unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = "docs/adr""#).unwrap();

        let discovered = discover(&subdir).unwrap();
        assert_eq!(discovered.root, temp.path());
        assert_eq!(discovered.config.adr_dir, PathBuf::from("docs/adr"));
    }

    #[test]
    fn test_discover_stops_at_git_root() {
        let temp = TempDir::new().unwrap();

        // Create a git repo structure
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        let subdir = temp.path().join("src");
        std::fs::create_dir(&subdir).unwrap();

        // Put config above git root (should not be found)
        // This test verifies we stop at .git

        let result = discover(&subdir);
        // Should return defaults since no config found within git repo
        assert!(result.is_ok());
        let discovered = result.unwrap();
        assert!(matches!(discovered.source, ConfigSource::Default));
    }

    #[test]
    fn test_discover_prefers_adrs_toml_over_adr_dir() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "legacy").unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = "modern""#).unwrap();

        let discovered = discover(temp.path()).unwrap();
        assert_eq!(discovered.config.adr_dir, PathBuf::from("modern"));
    }

    #[test]
    fn test_discover_finds_default_adr_dir() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("doc/adr")).unwrap();

        let discovered = discover(temp.path()).unwrap();
        assert_eq!(discovered.root, temp.path());
        assert_eq!(discovered.config.adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_discover_returns_defaults_when_nothing_found() {
        let temp = TempDir::new().unwrap();
        // Create .git to stop search
        std::fs::create_dir(temp.path().join(".git")).unwrap();

        let discovered = discover(temp.path()).unwrap();
        assert!(matches!(discovered.source, ConfigSource::Default));
        assert_eq!(discovered.config.adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_config_source_variants() {
        // Test that ConfigSource can be compared
        let project = ConfigSource::Project(PathBuf::from("test"));
        let global = ConfigSource::Global(PathBuf::from("test"));
        let env = ConfigSource::Environment;
        let default = ConfigSource::Default;

        assert_ne!(project, global);
        assert_ne!(env, default);
        assert_eq!(default, ConfigSource::Default);
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config::default();
        let other = Config {
            adr_dir: PathBuf::from("custom"),
            mode: ConfigMode::NextGen,
            templates: TemplateConfig {
                format: Some("madr".to_string()),
                variant: None,
                custom: None,
            },
        };

        base.merge(&other);
        assert_eq!(base.adr_dir, PathBuf::from("custom"));
        assert_eq!(base.mode, ConfigMode::NextGen);
        assert_eq!(base.templates.format, Some("madr".to_string()));
    }

    #[test]
    fn test_config_merge_preserves_default_adr_dir() {
        let mut base = Config {
            adr_dir: PathBuf::from("original"),
            ..Default::default()
        };
        let other = Config::default(); // has default adr_dir

        base.merge(&other);
        // Should keep original since other has default
        assert_eq!(base.adr_dir, PathBuf::from("original"));
    }

    // ========== Config Validation Tests ==========

    #[test]
    fn test_load_empty_adr_dir_in_toml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = """#).unwrap();

        let result = Config::load(temp.path());
        assert!(
            result.is_err(),
            "Empty adr_dir in TOML should produce an error"
        );
    }

    #[test]
    fn test_load_whitespace_only_adr_dir_file() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "   \n  ").unwrap();

        // With figment, whitespace-only .adr-dir returns empty map,
        // so we fall back to defaults, but the file exists so load succeeds
        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_invalid_format_string_accepted_in_toml() {
        // Invalid format strings are stored as-is in config; they only error
        // when parsed at ADR creation time. This is by design — the config
        // layer stores strings, the command layer validates them.
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "doc/adr"

[templates]
format = "nonexistent"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.templates.format, Some("nonexistent".to_string()));
    }

    #[test]
    fn test_invalid_variant_string_accepted_in_toml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "doc/adr"

[templates]
variant = "bogus"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.templates.variant, Some("bogus".to_string()));
    }

    #[test]
    fn test_invalid_mode_string_rejected() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"mode = "invalid_mode""#).unwrap();

        let result = Config::load(temp.path());
        assert!(
            result.is_err(),
            "Invalid mode should produce a TOML parse error"
        );
    }

    #[test]
    fn test_unknown_toml_fields_accepted() {
        // Serde's default behavior: unknown fields are silently ignored.
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "doc/adr"
unknown_field = "hello"

[templates]
also_unknown = true
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_custom_template_path_in_config() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("adrs.toml"),
            r#"
adr_dir = "doc/adr"
mode = "ng"

[templates]
custom = "templates/my-adr.md"
"#,
        )
        .unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(
            config.templates.custom,
            Some(PathBuf::from("templates/my-adr.md"))
        );
    }

    // ========== Save/Load Roundtrip Tests ==========

    #[test]
    fn test_save_and_load_roundtrip_nextgen_with_templates() {
        let temp = TempDir::new().unwrap();
        let original = Config {
            adr_dir: PathBuf::from("docs/decisions"),
            mode: ConfigMode::NextGen,
            templates: TemplateConfig {
                format: Some("madr".to_string()),
                variant: Some("minimal".to_string()),
                custom: Some(PathBuf::from("templates/custom.md")),
            },
        };

        original.save(temp.path()).unwrap();
        let loaded = Config::load(temp.path()).unwrap();

        assert_eq!(loaded.adr_dir, PathBuf::from("docs/decisions"));
        assert_eq!(loaded.mode, ConfigMode::NextGen);
        assert_eq!(loaded.templates.format, Some("madr".to_string()));
        assert_eq!(loaded.templates.variant, Some("minimal".to_string()));
        assert_eq!(
            loaded.templates.custom,
            Some(PathBuf::from("templates/custom.md"))
        );
    }

    #[test]
    fn test_save_and_load_roundtrip_nextgen_mode_serializes_as_ng() {
        // NextGen serializes as "ng" but should load back as NextGen
        let temp = TempDir::new().unwrap();
        let original = Config {
            mode: ConfigMode::NextGen,
            ..Default::default()
        };

        original.save(temp.path()).unwrap();

        // Verify the file contains "ng" not "nextgen"
        let content = std::fs::read_to_string(temp.path().join("adrs.toml")).unwrap();
        assert!(content.contains(r#"mode = "ng""#));

        // Load it back
        let loaded = Config::load(temp.path()).unwrap();
        assert_eq!(loaded.mode, ConfigMode::NextGen);
    }

    // ========== Config Merge Validation Tests ==========

    #[test]
    fn test_config_merge_variant_field() {
        let mut base = Config::default();
        let other = Config {
            templates: TemplateConfig {
                format: None,
                variant: Some("minimal".to_string()),
                custom: None,
            },
            ..Default::default()
        };

        base.merge(&other);
        assert_eq!(base.templates.variant, Some("minimal".to_string()));
    }

    #[test]
    fn test_config_merge_custom_field() {
        let mut base = Config::default();
        let other = Config {
            templates: TemplateConfig {
                format: None,
                variant: None,
                custom: Some(PathBuf::from("my-template.md")),
            },
            ..Default::default()
        };

        base.merge(&other);
        assert_eq!(base.templates.custom, Some(PathBuf::from("my-template.md")));
    }

    #[test]
    fn test_config_merge_does_not_overwrite_with_none() {
        let mut base = Config {
            templates: TemplateConfig {
                format: Some("madr".to_string()),
                variant: Some("minimal".to_string()),
                custom: Some(PathBuf::from("template.md")),
            },
            ..Default::default()
        };
        let other = Config::default(); // all template fields are None

        base.merge(&other);

        // None values in other should NOT overwrite existing values
        assert_eq!(base.templates.format, Some("madr".to_string()));
        assert_eq!(base.templates.variant, Some("minimal".to_string()));
        assert_eq!(base.templates.custom, Some(PathBuf::from("template.md")));
    }

    // ========== Figment-specific Tests ==========

    #[test]
    fn test_figment_layers_toml_over_adr_dir() {
        let temp = TempDir::new().unwrap();
        // Create both - toml should win
        std::fs::write(temp.path().join(".adr-dir"), "legacy-path").unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = "toml-path""#).unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("toml-path"));
    }

    #[test]
    fn test_find_project_root_basic() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("adrs.toml"), r#"adr_dir = "test""#).unwrap();

        let result = find_project_root(temp.path());
        assert!(result.is_some());
        let (root, source) = result.unwrap();
        assert_eq!(root, temp.path());
        assert!(matches!(source, ConfigSource::Project(_)));
    }

    #[test]
    fn test_find_project_root_none_when_empty() {
        let temp = TempDir::new().unwrap();
        // Create .git to stop search but no config
        std::fs::create_dir(temp.path().join(".git")).unwrap();

        let result = find_project_root(temp.path());
        assert!(result.is_none());
    }
}
