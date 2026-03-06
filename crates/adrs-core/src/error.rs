//! # Error Types
//!
//! Error types and result aliases for `adrs-core` operations.
//!
//! ## Overview
//!
//! All fallible operations in this crate return [`Result<T>`], which is
//! an alias for `std::result::Result<T, Error>`.
//!
//! ## Error Variants
//!
//! | Variant | Cause | Recovery |
//! |---------|-------|----------|
//! | [`Error::AdrDirNotFound`] | No `.adr-dir` or `adrs.toml` | Run `adrs init` |
//! | [`Error::AdrNotFound`] | ADR doesn't exist | Check valid numbers with `list` |
//! | [`Error::InvalidFormat`] | Malformed ADR content | Fix file syntax |
//! | [`Error::Io`] | File system error | Check path and permissions |
//! | [`Error::TemplateError`] | Template rendering failed | Check template syntax |
//!
//! ## Examples
//!
//! ### Handling specific errors
//!
//! ```rust
//! use adrs_core::{Repository, Error};
//!
//! fn open_repo(path: &str) -> Result<(), String> {
//!     match Repository::open(path) {
//!         Ok(repo) => {
//!             println!("Opened repository with {} ADRs", repo.list().unwrap().len());
//!             Ok(())
//!         }
//!         Err(Error::AdrDirNotFound) => {
//!             Err("No repository found. Run 'adrs init' first.".into())
//!         }
//!         Err(e) => Err(format!("Failed to open repository: {}", e)),
//!     }
//! }
//!
//! // This path doesn't exist, so we get AdrDirNotFound
//! let result = open_repo("/nonexistent/path");
//! assert!(result.is_err());
//! ```
//!
//! ### Converting to standard errors
//!
//! ```rust
//! use adrs_core::{Repository, Error};
//!
//! fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
//!     // Error implements std::error::Error, so it works with ?
//!     let result = Repository::open("/nonexistent");
//!     assert!(result.is_err());
//!     Ok(())
//! }
//! # example().unwrap();
//! ```

use std::path::PathBuf;

/// A specialized Result type for ADR operations.
///
/// This is defined as `std::result::Result<T, Error>` for convenience.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during ADR operations.
///
/// This enum represents all possible errors returned by `adrs-core`
/// functions. Each variant includes context about what went wrong.
///
/// # Display
///
/// All variants implement [`Display`](std::fmt::Display) with
/// human-readable messages suitable for showing to users.
///
/// # Source
///
/// Variants that wrap other errors (like [`Io`](Self::Io)) provide
/// access to the underlying error via [`std::error::Error::source`].
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
