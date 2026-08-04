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
    AmbiguousAdr {
        /// The query string that matched more than one ADR.
        query: String,
        /// Human-readable identifiers of the ADRs that matched.
        matches: Vec<String>,
    },

    /// Invalid ADR number.
    #[error("Invalid ADR number: {0}")]
    InvalidNumber(String),

    /// Invalid ADR format (parsing failed).
    #[error("Invalid ADR format in {path}: {reason}")]
    InvalidFormat {
        /// Path to the ADR file that failed to parse.
        path: PathBuf,
        /// Why parsing failed.
        reason: String,
    },

    /// Missing required field in ADR.
    #[error("Missing required field '{field}' in {path}")]
    MissingField {
        /// Path to the ADR file missing the field.
        path: PathBuf,
        /// Name of the required field that was missing.
        field: String,
    },

    /// Invalid status value.
    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    /// Invalid link format.
    #[error("Invalid link format: {0}")]
    InvalidLink(String),

    /// `Repository::renumber`'s `from` number matched more than one record
    /// and no `--file` was given to disambiguate.
    #[error(
        "ADR {number} is ambiguous: {} records are numbered {number} ({candidates:?}). Use --file <path> to select one.",
        candidates.len()
    )]
    AmbiguousRenumberSource {
        /// The ambiguous source number.
        number: u32,
        /// Paths of every record numbered `number`.
        candidates: Vec<String>,
    },

    /// `Repository::renumber`'s `--file` did not match any record numbered `from`.
    #[error("{file} does not match any ADR numbered {number}. Candidates: {candidates:?}")]
    RenumberFileMismatch {
        /// The source number `--file` was expected to disambiguate.
        number: u32,
        /// The `--file` path given.
        file: PathBuf,
        /// Paths of every record numbered `number`.
        candidates: Vec<String>,
    },

    /// `Repository::renumber`'s `to` number is already used by another record.
    #[error(
        "ADR {to} is already used by '{occupant_title}' ({occupant_path}); try {suggestion} instead"
    )]
    RenumberTargetOccupied {
        /// The requested destination number.
        to: u32,
        /// Title of the record currently occupying `to`.
        occupant_title: String,
        /// Path of the record currently occupying `to`.
        occupant_path: PathBuf,
        /// The smallest free number, offered as a suggestion.
        suggestion: u32,
    },

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
    Yaml(#[from] serde_yaml_neo::Error),

    /// TOML parsing error.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}
