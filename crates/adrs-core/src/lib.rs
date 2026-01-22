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

mod config;
pub mod doctor;
mod error;
pub mod export;
mod parse;
mod repository;
mod template;
mod types;

pub use config::{Config, ConfigMode, ConfigSource, DiscoveredConfig, discover};
pub use doctor::{Check, Diagnostic, DoctorReport, Severity, check as doctor_check};
pub use error::{Error, Result};
pub use export::{
    ConsideredOption, JSON_ADR_SCHEMA, JSON_ADR_VERSION, JsonAdr, JsonAdrBulkExport, JsonAdrLink,
    RepositoryInfo, ToolInfo, export_adr, export_repository,
};
pub use parse::Parser;
pub use repository::Repository;
pub use template::{Template, TemplateEngine, TemplateFormat, TemplateVariant};
pub use types::{Adr, AdrLink, AdrStatus, LinkKind};
