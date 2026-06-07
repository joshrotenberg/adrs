//! # adrs-core
//!
//! The engine behind the [`adrs`] command-line tool: a standalone library for
//! creating, reading, linking, validating, and exporting Architecture Decision
//! Records (ADRs).
//!
//! Depend on `adrs-core` directly when you want to manage ADRs from your own
//! program -- a build tool, an editor integration, or an AI agent -- instead of
//! shelling out to the CLI. The CLI is a thin layer over this crate, so anything
//! `adrs` does is available here as a normal Rust API.
//!
//! [`adrs`]: https://crates.io/crates/adrs
//!
//! ## Key types
//!
//! - [`Repository`] -- the main entry point. Open or initialize a directory of
//!   ADRs, then [`list`](Repository::list), [`new_adr`](Repository::new_adr),
//!   [`link`](Repository::link), [`supersede`](Repository::supersede), and
//!   [`set_status`](Repository::set_status).
//! - [`Adr`] -- a single decision record: number, title, [`AdrStatus`], body
//!   sections, [`AdrLink`]s, tags, and MADR metadata (deciders, consulted,
//!   informed).
//! - [`AdrStatus`] and [`LinkKind`] -- typed status values and link
//!   relationships (e.g. `Supersedes` / `SupersededBy`).
//! - [`Config`] and [`discover`] -- repository configuration and upward
//!   directory discovery (find the ADR directory from a nested path).
//! - [`Parser`] -- parse an ADR from markdown, in either legacy (adr-tools) or
//!   YAML-frontmatter form.
//! - The [`export`] module -- convert ADRs to and from the **JSON-ADR**
//!   interchange format, handy for feeding a decision log to other tools or to
//!   an AI agent that reasons over it.
//! - The [`lint`] module -- repository health checks via [`check_all`]:
//!   broken links, duplicate numbers, numbering gaps, and missing fields.
//! - [`Error`] and [`Result`] -- the library's error type, built on `thiserror`.
//!
//! ## Modes
//!
//! ADRs can be stored in two on-disk formats:
//!
//! - **Compatible mode** (default): markdown-only, interoperable with
//!   [adr-tools]. Reads both legacy and next-gen files; writes legacy.
//! - **Next-gen mode**: YAML frontmatter for structured metadata, enabling
//!   richer features like typed links, tags, and stronger validation.
//!
//! The mode is chosen at [`Repository::init`] time (the `ng` argument) or via
//! [`Config`], and the parser transparently reads either format.
//!
//! [adr-tools]: https://github.com/npryce/adr-tools
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
//! ## Exporting to JSON-ADR
//!
//! [`export_repository`] converts an open [`Repository`] to the JSON-ADR format.
//! For a plain directory that isn't an initialized adrs repository, use
//! [`export_directory`] (or [`export_directory_with_warnings`] to also receive a
//! message for every file that failed to parse, rather than skipping silently).
//!
//! ## More examples
//!
//! The [`examples`](https://github.com/joshrotenberg/adrs/tree/main/crates/adrs-core/examples)
//! directory has runnable programs: `create_and_list`, `link_adrs`,
//! `export_json`, and `lint_repository`.

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
