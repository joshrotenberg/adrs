//! # adrs-core
//!
//! Core library for managing Architecture Decision Records (ADRs).
//!
//! This crate provides the foundational types and operations for working
//! with ADRs, including parsing, creating, linking, and querying decision
//! records. It serves as the backend for the `adrs` CLI tool.
//!
//! ## Quick Start
//!
//! ```rust
//! use adrs_core::{Repository, Result};
//! use tempfile::TempDir;
//!
//! fn main() -> Result<()> {
//!     // Create a temporary repository for this example
//!     let temp = TempDir::new().unwrap();
//!     let repo = Repository::init(temp.path(), None, false)?;
//!
//!     // Create a new ADR
//!     let (adr, path) = repo.new_adr("Use Rust for implementation")?;
//!     println!("Created ADR #{}: {}", adr.number, path.display());
//!
//!     // List all ADRs
//!     for adr in repo.list()? {
//!         println!("{:04}. {} [{}]", adr.number, adr.title, adr.status);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Core Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Repository`] | Main entry point for ADR operations |
//! | [`Adr`] | A single Architecture Decision Record |
//! | [`AdrStatus`] | Lifecycle status (Proposed, Accepted, etc.) |
//! | [`AdrLink`] | Relationship between two ADRs |
//! | [`Config`] | Repository configuration |
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`export`] | JSON-ADR format import/export |
//! | [`lint`] | Validation and linting |
//!
//! ## Modes
//!
//! The library supports two operational modes:
//!
//! | Mode | Config File | Frontmatter | Compatibility |
//! |------|-------------|-------------|---------------|
//! | **Compatible** | `.adr-dir` | None | adr-tools |
//! | **NextGen** | `adrs.toml` | YAML | Full features |
//!
//! See [`ConfigMode`] to check or set the mode.
//!
//! ## Error Handling
//!
//! All fallible operations return [`Result<T>`], which uses [`Error`]
//! for error variants. See the [`Error`] type for details.

mod config;
#[doc(hidden)]
pub mod doctest_helpers;
pub mod doctor;
mod error;
pub mod export;
pub mod lint;
mod parse;
mod repository;
mod template;
mod types;

pub use config::{
    AdrDirFile, AdrsConfigEnv, AdrsEnv, Config, ConfigMode, ConfigSource, ConfigWriteTarget,
    DiscoveredConfig, GitConfig, GitConfigScope, discover, global_config_dir,
};
pub use error::{Error, Result};
pub use export::{
    ConsideredOption, ImportOptions, ImportResult, JSON_ADR_SCHEMA, JSON_ADR_VERSION, JsonAdr,
    JsonAdrBulkExport, JsonAdrLink, JsonAdrSingle, RepositoryInfo, ToolInfo, export_adr,
    export_directory, export_repository, import_to_directory,
};
pub use lint::{Issue, IssueSeverity, LintReport, check_all, check_repository, lint_adr, lint_all};
pub use parse::Parser;
pub use repository::Repository;
pub use template::{Template, TemplateEngine, TemplateFormat, TemplateVariant};
pub use types::{Adr, AdrLink, AdrStatus, LinkKind};

// Legacy doctor module - deprecated, use lint module instead
#[deprecated(since = "0.6.0", note = "Use lint module instead")]
pub use doctor::{Check, Diagnostic, DoctorReport, Severity, check as doctor_check};
