//! Configuration handling for ADR repositories.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default ADR directory name.
pub const DEFAULT_ADR_DIR: &str = "doc/adr";

/// Legacy configuration file name (adr-tools compatible).
pub const LEGACY_CONFIG_FILE: &str = ".adr-dir";

/// New configuration file name.
pub const CONFIG_FILE: &str = "adrs.toml";

/// Configuration for an ADR repository.
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
    /// Searches for configuration in the following order:
    /// 1. `adrs.toml` (new format)
    /// 2. `.adr-dir` (legacy adr-tools format)
    /// 3. Default configuration
    pub fn load(root: &Path) -> Result<Self> {
        // Try new config first
        let config_path = root.join(CONFIG_FILE);
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(config);
        }

        // Try legacy .adr-dir file
        let legacy_path = root.join(LEGACY_CONFIG_FILE);
        if legacy_path.exists() {
            let adr_dir = std::fs::read_to_string(&legacy_path)?.trim().to_string();
            return Ok(Self {
                adr_dir: PathBuf::from(adr_dir),
                mode: ConfigMode::Compatible,
                templates: TemplateConfig::default(),
            });
        }

        // Check if default directory exists
        let default_dir = root.join(DEFAULT_ADR_DIR);
        if default_dir.exists() {
            return Ok(Self::default());
        }

        Err(Error::AdrDirNotFound)
    }

    /// Load configuration, or return default if not found.
    pub fn load_or_default(root: &Path) -> Self {
        Self::load(root).unwrap_or_default()
    }

    /// Save configuration to the given directory.
    pub fn save(&self, root: &Path) -> Result<()> {
        match self.mode {
            ConfigMode::Compatible => {
                // Write legacy .adr-dir file
                let path = root.join(LEGACY_CONFIG_FILE);
                std::fs::write(&path, self.adr_dir.display().to_string())?;
            }
            ConfigMode::NextGen => {
                // Write adrs.toml
                let path = root.join(CONFIG_FILE);
                let content =
                    toml::to_string_pretty(self).map_err(|e| Error::ConfigError(e.to_string()))?;
                std::fs::write(&path, content)?;
            }
        }
        Ok(())
    }

    /// Returns the full path to the ADR directory.
    pub fn adr_path(&self, root: &Path) -> PathBuf {
        root.join(&self.adr_dir)
    }

    /// Returns true if running in next-gen mode.
    pub fn is_next_gen(&self) -> bool {
        matches!(self.mode, ConfigMode::NextGen)
    }
}

/// The mode of operation for the ADR tool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigMode {
    /// Compatible with adr-tools (markdown-only, no frontmatter).
    #[default]
    Compatible,

    /// Next-gen mode with YAML frontmatter and enhanced features.
    #[serde(rename = "ng")]
    NextGen,
}

/// Template configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateConfig {
    /// The default template format to use.
    pub format: Option<String>,

    /// Path to a custom template file.
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
                custom: None,
            },
        };

        original.save(temp.path()).unwrap();
        let loaded = Config::load(temp.path()).unwrap();

        assert_eq!(loaded.adr_dir, original.adr_dir);
        assert_eq!(loaded.mode, ConfigMode::NextGen);
        assert_eq!(loaded.templates.format, Some("markdown".to_string()));
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

    // ========== TemplateConfig Tests ==========

    #[test]
    fn test_template_config_default() {
        let config = TemplateConfig::default();
        assert!(config.format.is_none());
        assert!(config.custom.is_none());
    }

    #[test]
    fn test_template_config_serialization() {
        let config = TemplateConfig {
            format: Some("nygard".to_string()),
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

        let config = Config::load(temp.path()).unwrap();
        // Empty string becomes empty path
        assert_eq!(config.adr_dir, PathBuf::from(""));
    }
}
