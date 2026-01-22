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

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.adr_dir, PathBuf::from("doc/adr"));
        assert_eq!(config.mode, ConfigMode::Compatible);
    }

    #[test]
    fn test_load_legacy_config() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".adr-dir"), "decisions").unwrap();

        let config = Config::load(temp.path()).unwrap();
        assert_eq!(config.adr_dir, PathBuf::from("decisions"));
        assert_eq!(config.mode, ConfigMode::Compatible);
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
    }
}
