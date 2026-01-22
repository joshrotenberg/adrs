//! Error types for adrs-core.

use std::path::PathBuf;

/// Result type alias using the library's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when working with ADRs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The ADR directory was not found.
    #[error("ADR directory not found. Run 'adrs init' to create one.")]
    AdrDirNotFound,

    /// The ADR directory already exists.
    #[error("ADR directory already exists: {0}")]
    AdrDirExists(PathBuf),

    /// An ADR was not found.
    #[error("ADR not found: {0}")]
    AdrNotFound(String),

    /// Multiple ADRs matched a query.
    #[error("Multiple ADRs match '{query}': {matches:?}")]
    AmbiguousAdr { query: String, matches: Vec<String> },

    /// Invalid ADR number.
    #[error("Invalid ADR number: {0}")]
    InvalidNumber(String),

    /// Invalid ADR format (parsing failed).
    #[error("Invalid ADR format in {path}: {reason}")]
    InvalidFormat { path: PathBuf, reason: String },

    /// Missing required field in ADR.
    #[error("Missing required field '{field}' in {path}")]
    MissingField { path: PathBuf, field: String },

    /// Invalid status value.
    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    /// Invalid link format.
    #[error("Invalid link format: {0}")]
    InvalidLink(String),

    /// Template not found.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Template rendering error.
    #[error("Template error: {0}")]
    TemplateError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// TOML parsing error.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}
