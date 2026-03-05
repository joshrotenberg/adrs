//! Custom Figment providers for ADR configuration.
//!
//! This module provides custom configuration providers that integrate with
//! figment's layered configuration system.

use figment::value::{Dict, Map, Value};
use figment::{Metadata, Profile, Provider};
use std::path::PathBuf;

/// Provider that reads legacy `.adr-dir` files.
///
/// The `.adr-dir` file format is a single line containing the path to the
/// ADR directory, compatible with the original adr-tools.
///
/// # Example
///
/// ```text
/// doc/adr
/// ```
///
/// This provider reads the file and maps it to `{ adr_dir: "<contents>" }`.
#[derive(Debug, Clone)]
pub struct AdrDirFile {
    path: PathBuf,
}

impl AdrDirFile {
    /// Create a new provider for the given `.adr-dir` file path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Provider for AdrDirFile {
    fn metadata(&self) -> Metadata {
        Metadata::named("adr-dir file").source(self.path.display().to_string())
    }

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
