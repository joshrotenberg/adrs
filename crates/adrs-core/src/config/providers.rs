//! Custom Figment providers for ADR configuration.
//!
//! This module provides custom configuration providers that integrate with
//! [figment](https://docs.rs/figment)'s layered configuration system.
//!
//! # Providers
//!
//! | Provider | Source | Priority |
//! |----------|--------|----------|
//! | [`AdrDirFile`] | `.adr-dir` file | Layer 3 (legacy) |
//!
//! # Architecture
//!
//! These providers implement [`figment::Provider`] to participate in
//! figment's configuration merging. Each provider returns a
//! `Map<Profile, Dict>` containing its configuration values.
//!
//! When a file doesn't exist or is empty, providers return an empty map
//! (not an error), allowing higher-priority sources to provide values.

use figment::value::{Dict, Map, Value};
use figment::{Metadata, Profile, Provider};
use std::path::PathBuf;

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
