//! Core types for representing ADRs.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::Date;

/// An Architecture Decision Record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adr {
    /// The ADR number (e.g., 1, 2, 3).
    pub number: u32,

    /// The title of the decision.
    pub title: String,

    /// The date the decision was made.
    #[serde(with = "date_format")]
    pub date: Date,

    /// The current status of the decision.
    pub status: AdrStatus,

    /// Links to other ADRs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<AdrLink>,

    /// The context section (why this decision was needed).
    #[serde(skip)]
    pub context: String,

    /// The decision section (what was decided).
    #[serde(skip)]
    pub decision: String,

    /// The consequences section (what happens as a result).
    #[serde(skip)]
    pub consequences: String,

    /// Path to the ADR file (not serialized to frontmatter).
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Adr {
    /// Create a new ADR with the given number and title.
    pub fn new(number: u32, title: impl Into<String>) -> Self {
        Self {
            number,
            title: title.into(),
            date: crate::parse::today(),
            status: AdrStatus::Proposed,
            links: Vec::new(),
            context: String::new(),
            decision: String::new(),
            consequences: String::new(),
            path: None,
        }
    }

    /// Returns the formatted filename for this ADR.
    pub fn filename(&self) -> String {
        format!("{:04}-{}.md", self.number, slug(&self.title))
    }

    /// Returns the full title with number prefix (e.g., "1. Use Rust").
    pub fn full_title(&self) -> String {
        format!("{}. {}", self.number, self.title)
    }

    /// Add a link to another ADR.
    pub fn add_link(&mut self, link: AdrLink) {
        self.links.push(link);
    }

    /// Set the status and optionally record what this supersedes.
    pub fn set_status(&mut self, status: AdrStatus) {
        self.status = status;
    }
}

/// The status of an ADR.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdrStatus {
    /// The decision is proposed but not yet accepted.
    #[default]
    Proposed,

    /// The decision has been accepted.
    Accepted,

    /// The decision has been deprecated.
    Deprecated,

    /// The decision has been superseded by another.
    Superseded,

    /// Custom status (for compatibility with other tools).
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "Proposed"),
            Self::Accepted => write!(f, "Accepted"),
            Self::Deprecated => write!(f, "Deprecated"),
            Self::Superseded => write!(f, "Superseded"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for AdrStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "proposed" => Self::Proposed,
            "accepted" => Self::Accepted,
            "deprecated" => Self::Deprecated,
            "superseded" => Self::Superseded,
            _ => Self::Custom(s.to_string()),
        })
    }
}

/// A link between ADRs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdrLink {
    /// The target ADR number.
    pub target: u32,

    /// The kind of link.
    pub kind: LinkKind,

    /// Optional description of the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AdrLink {
    /// Create a new link to another ADR.
    pub fn new(target: u32, kind: LinkKind) -> Self {
        Self {
            target,
            kind,
            description: None,
        }
    }

    /// Create a new link with a description.
    pub fn with_description(target: u32, kind: LinkKind, description: impl Into<String>) -> Self {
        Self {
            target,
            kind,
            description: Some(description.into()),
        }
    }
}

/// The kind of link between ADRs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    /// This ADR supersedes the target.
    Supersedes,

    /// This ADR is superseded by the target.
    SupersededBy,

    /// This ADR amends the target.
    Amends,

    /// This ADR is amended by the target.
    AmendedBy,

    /// This ADR relates to the target.
    RelatesTo,

    /// Custom link type (for extensibility).
    #[serde(untagged)]
    Custom(String),
}

impl std::fmt::Display for LinkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Supersedes => write!(f, "Supersedes"),
            Self::SupersededBy => write!(f, "Superseded by"),
            Self::Amends => write!(f, "Amends"),
            Self::AmendedBy => write!(f, "Amended by"),
            Self::RelatesTo => write!(f, "Relates to"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl std::str::FromStr for LinkKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "supersedes" => Self::Supersedes,
            "superseded by" | "superseded-by" => Self::SupersededBy,
            "amends" => Self::Amends,
            "amended by" | "amended-by" => Self::AmendedBy,
            "relates to" | "relates-to" => Self::RelatesTo,
            _ => Self::Custom(s.to_string()),
        })
    }
}

/// Convert a title to a URL-safe slug.
fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Custom date serialization format (YYYY-MM-DD).
mod date_format {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::Date;
    use time::format_description::well_known::Iso8601;

    pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .format(&Iso8601::DATE)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Date::parse(&s, &Iso8601::DATE).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug() {
        assert_eq!(slug("Use Rust"), "use-rust");
        assert_eq!(slug("Use React for Frontend"), "use-react-for-frontend");
        assert_eq!(slug("API v2.0 Design"), "api-v2-0-design");
        assert_eq!(slug("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_adr_filename() {
        let adr = Adr::new(1, "Use Rust");
        assert_eq!(adr.filename(), "0001-use-rust.md");

        let adr = Adr::new(42, "API Design Guidelines");
        assert_eq!(adr.filename(), "0042-api-design-guidelines.md");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(AdrStatus::Accepted.to_string(), "Accepted");
        assert_eq!(AdrStatus::Custom("Draft".into()).to_string(), "Draft");
    }

    #[test]
    fn test_status_parse() {
        assert_eq!(
            "accepted".parse::<AdrStatus>().unwrap(),
            AdrStatus::Accepted
        );
        assert_eq!(
            "PROPOSED".parse::<AdrStatus>().unwrap(),
            AdrStatus::Proposed
        );
        assert_eq!(
            "custom-status".parse::<AdrStatus>().unwrap(),
            AdrStatus::Custom("custom-status".into())
        );
    }

    #[test]
    fn test_link_kind_display() {
        assert_eq!(LinkKind::Supersedes.to_string(), "Supersedes");
        assert_eq!(LinkKind::SupersededBy.to_string(), "Superseded by");
    }
}
