//! # adrs-core
//!
//! Core library for managing Architecture Decision Records (ADRs).
//!
//! This library provides the foundational types and operations for working with ADRs,
//! including parsing, creating, linking, and querying decision records.
//!
//! ## Modes
//!
//! The library supports two modes:
//!
//! - **Compatible mode** (default): Writes markdown-only format compatible with adr-tools,
//!   but can read both legacy and next-gen formats.
//! - **Next-gen mode**: Uses YAML frontmatter for structured metadata, enabling richer
//!   features like typed links and better validation.
//!
//! ## Quick start
//!
//! ```
//! use adrs_core::Repository;
//!
//! # fn main() -> adrs_core::Result<()> {
//! # let tmp = tempfile::tempdir().unwrap();
//! // Initialize a repository (compatible mode).
//! let repo = Repository::init(tmp.path(), None, false)?;
//!
//! // Create a decision record.
//! let (adr, _path) = repo.new_adr("Use PostgreSQL for persistence")?;
//! assert_eq!(adr.title, "Use PostgreSQL for persistence");
//!
//! // List all ADRs (`init` seeds the first one).
//! let all = repo.list()?;
//! assert!(all.len() >= 2);
//! # Ok(())
//! # }
//! ```
//!
//! See the [`examples`](https://github.com/joshrotenberg/adrs/tree/main/crates/adrs-core/examples)
//! directory for more, including linking ADRs, exporting to JSON-ADR, and linting.

mod config;
pub mod doctor;
mod error;
pub mod export;
pub mod lint;
mod parse;
mod repository;
mod template;
mod types;

pub use config::{Config, ConfigMode, ConfigSource, DiscoveredConfig, discover};
pub use error::{Error, Result};
pub use export::{
    ConsideredOption, ImportOptions, ImportResult, JSON_ADR_SCHEMA, JSON_ADR_VERSION, JsonAdr,
    JsonAdrBulkExport, JsonAdrLink, JsonAdrSingle, RepositoryInfo, ToolInfo, export_adr,
    export_directory, export_directory_with_warnings, export_repository, import_to_directory,
};
pub use lint::{Issue, IssueSeverity, LintReport, check_all, check_repository, lint_adr, lint_all};
pub use parse::Parser;
pub use repository::Repository;
pub use template::{Template, TemplateEngine, TemplateFormat, TemplateVariant};
pub use types::{Adr, AdrLink, AdrStatus, LinkKind};

// Legacy doctor module - deprecated, use lint module instead
#[deprecated(since = "0.6.0", note = "Use lint module instead")]
pub use doctor::{Check, Diagnostic, DoctorReport, Severity, check as doctor_check};
