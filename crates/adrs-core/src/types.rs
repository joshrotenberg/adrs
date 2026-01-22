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
            // Support both correct spelling and adr-tools typo
            "superseded" | "superceded" => Self::Superseded,
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
            // Support both correct spelling and adr-tools typo
            "supersedes" | "supercedes" => Self::Supersedes,
            "superseded by" | "superseded-by" | "superceded by" | "superceded-by" => {
                Self::SupersededBy
            }
            "amends" => Self::Amends,
            "amended by" | "amended-by" => Self::AmendedBy,
            "relates to" | "relates-to" => Self::RelatesTo,
            _ => Self::Custom(s.to_string()),
        })
    }
}

/// Convert a title to a URL-safe slug.
///
/// Only ASCII alphanumeric characters are preserved; everything else becomes a dash.
/// Consecutive dashes are collapsed, and leading/trailing dashes are removed.
fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
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
    use proptest::prelude::*;
    use test_case::test_case;
    use time::Month;

    // ========== Slug Tests ==========

    #[test]
    fn test_slug_basic() {
        assert_eq!(slug("Use Rust"), "use-rust");
        assert_eq!(slug("Use React for Frontend"), "use-react-for-frontend");
    }

    #[test_case("API v2.0 Design" => "api-v2-0-design"; "version numbers")]
    #[test_case("  Multiple   Spaces  " => "multiple-spaces"; "multiple spaces")]
    #[test_case("CamelCase" => "camelcase"; "camel case")]
    #[test_case("kebab-case" => "kebab-case"; "already kebab")]
    #[test_case("snake_case" => "snake-case"; "snake to kebab")]
    #[test_case("MixedCase-and_stuff" => "mixedcase-and-stuff"; "mixed")]
    #[test_case("" => ""; "empty string")]
    #[test_case("---" => ""; "only dashes")]
    #[test_case("Hello, World!" => "hello-world"; "punctuation")]
    #[test_case("C++ vs Rust" => "c-vs-rust"; "special chars")]
    #[test_case("100% Complete" => "100-complete"; "percentage")]
    #[test_case("Über Design" => "ber-design"; "unicode umlaut")]
    #[test_case("日本語" => ""; "japanese characters")]
    #[test_case("café" => "caf"; "accented characters")]
    fn test_slug_cases(input: &str) -> String {
        slug(input)
    }

    proptest! {
        #[test]
        fn test_slug_never_starts_or_ends_with_dash(s in "[a-zA-Z0-9 ]{1,50}") {
            let result = slug(&s);
            if !result.is_empty() {
                prop_assert!(!result.starts_with('-'), "Slug starts with dash: {}", result);
                prop_assert!(!result.ends_with('-'), "Slug ends with dash: {}", result);
            }
        }

        #[test]
        fn test_slug_no_consecutive_dashes(s in "\\PC{1,100}") {
            let result = slug(&s);
            prop_assert!(!result.contains("--"), "Slug has consecutive dashes: {}", result);
        }

        #[test]
        fn test_slug_only_lowercase_alphanumeric_and_dashes(s in "\\PC{1,100}") {
            let result = slug(&s);
            for c in result.chars() {
                prop_assert!(
                    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-',
                    "Invalid character in slug: {} (from {})", c, result
                );
            }
        }
    }

    // ========== Adr Tests ==========

    #[test]
    fn test_adr_new() {
        let adr = Adr::new(1, "Use Rust");
        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Use Rust");
        assert_eq!(adr.status, AdrStatus::Proposed);
        assert!(adr.links.is_empty());
        assert!(adr.context.is_empty());
        assert!(adr.decision.is_empty());
        assert!(adr.consequences.is_empty());
        assert!(adr.path.is_none());
    }

    #[test_case(1, "Use Rust" => "0001-use-rust.md"; "single digit")]
    #[test_case(42, "API Design Guidelines" => "0042-api-design-guidelines.md"; "two digits")]
    #[test_case(999, "Last Decision" => "0999-last-decision.md"; "three digits")]
    #[test_case(9999, "Max Normal" => "9999-max-normal.md"; "four digits")]
    #[test_case(10000, "Beyond Four Digits" => "10000-beyond-four-digits.md"; "five digits")]
    fn test_adr_filename(number: u32, title: &str) -> String {
        Adr::new(number, title).filename()
    }

    #[test]
    fn test_adr_full_title() {
        let adr = Adr::new(1, "Use Rust");
        assert_eq!(adr.full_title(), "1. Use Rust");

        let adr = Adr::new(42, "API Design");
        assert_eq!(adr.full_title(), "42. API Design");
    }

    #[test]
    fn test_adr_add_link() {
        let mut adr = Adr::new(1, "Use Rust");
        adr.add_link(AdrLink::new(2, LinkKind::Supersedes));

        assert_eq!(adr.links.len(), 1);
        assert_eq!(adr.links[0].target, 2);
        assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
    }

    #[test]
    fn test_adr_set_status() {
        let mut adr = Adr::new(1, "Use Rust");
        assert_eq!(adr.status, AdrStatus::Proposed);

        adr.set_status(AdrStatus::Accepted);
        assert_eq!(adr.status, AdrStatus::Accepted);

        adr.set_status(AdrStatus::Superseded);
        assert_eq!(adr.status, AdrStatus::Superseded);
    }

    proptest! {
        #[test]
        fn test_adr_filename_always_ends_with_md(number in 1u32..100000, title in "[a-zA-Z ]{1,50}") {
            let adr = Adr::new(number, &title);
            prop_assert!(adr.filename().ends_with(".md"));
        }

        #[test]
        fn test_adr_filename_starts_with_padded_number(number in 1u32..10000, title in "[a-zA-Z]{1,10}") {
            let adr = Adr::new(number, &title);
            let filename = adr.filename();
            let prefix: String = filename.chars().take(4).collect();
            prop_assert_eq!(prefix, format!("{:04}", number));
        }
    }

    // ========== AdrStatus Tests ==========

    #[test_case(AdrStatus::Proposed => "Proposed")]
    #[test_case(AdrStatus::Accepted => "Accepted")]
    #[test_case(AdrStatus::Deprecated => "Deprecated")]
    #[test_case(AdrStatus::Superseded => "Superseded")]
    #[test_case(AdrStatus::Custom("Draft".into()) => "Draft")]
    #[test_case(AdrStatus::Custom("In Review".into()) => "In Review")]
    fn test_status_display(status: AdrStatus) -> String {
        status.to_string()
    }

    #[test_case("proposed" => AdrStatus::Proposed; "lowercase proposed")]
    #[test_case("PROPOSED" => AdrStatus::Proposed; "uppercase proposed")]
    #[test_case("Proposed" => AdrStatus::Proposed; "mixed case proposed")]
    #[test_case("accepted" => AdrStatus::Accepted; "lowercase accepted")]
    #[test_case("ACCEPTED" => AdrStatus::Accepted; "uppercase accepted")]
    #[test_case("deprecated" => AdrStatus::Deprecated; "lowercase deprecated")]
    #[test_case("superseded" => AdrStatus::Superseded; "lowercase superseded")]
    #[test_case("draft" => AdrStatus::Custom("draft".into()); "custom draft")]
    #[test_case("in-review" => AdrStatus::Custom("in-review".into()); "custom in review")]
    fn test_status_parse(input: &str) -> AdrStatus {
        input.parse().unwrap()
    }

    #[test]
    fn test_status_default() {
        assert_eq!(AdrStatus::default(), AdrStatus::Proposed);
    }

    #[test]
    fn test_status_roundtrip_standard() {
        for status in [
            AdrStatus::Proposed,
            AdrStatus::Accepted,
            AdrStatus::Deprecated,
            AdrStatus::Superseded,
        ] {
            let s = status.to_string();
            let parsed: AdrStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    // ========== LinkKind Tests ==========

    #[test_case(LinkKind::Supersedes => "Supersedes")]
    #[test_case(LinkKind::SupersededBy => "Superseded by")]
    #[test_case(LinkKind::Amends => "Amends")]
    #[test_case(LinkKind::AmendedBy => "Amended by")]
    #[test_case(LinkKind::RelatesTo => "Relates to")]
    #[test_case(LinkKind::Custom("Extends".into()) => "Extends")]
    fn test_link_kind_display(kind: LinkKind) -> String {
        kind.to_string()
    }

    #[test_case("supersedes" => LinkKind::Supersedes; "lowercase supersedes")]
    #[test_case("Supersedes" => LinkKind::Supersedes; "mixed case supersedes")]
    #[test_case("superseded by" => LinkKind::SupersededBy; "superseded by space")]
    #[test_case("superseded-by" => LinkKind::SupersededBy; "superseded by dash")]
    #[test_case("amends" => LinkKind::Amends; "amends")]
    #[test_case("amended by" => LinkKind::AmendedBy; "amended by space")]
    #[test_case("amended-by" => LinkKind::AmendedBy; "amended by dash")]
    #[test_case("relates to" => LinkKind::RelatesTo; "relates to space")]
    #[test_case("relates-to" => LinkKind::RelatesTo; "relates to dash")]
    #[test_case("extends" => LinkKind::Custom("extends".into()); "custom extends")]
    fn test_link_kind_parse(input: &str) -> LinkKind {
        input.parse().unwrap()
    }

    // ========== AdrLink Tests ==========

    #[test]
    fn test_adr_link_new() {
        let link = AdrLink::new(5, LinkKind::Supersedes);
        assert_eq!(link.target, 5);
        assert_eq!(link.kind, LinkKind::Supersedes);
        assert!(link.description.is_none());
    }

    #[test]
    fn test_adr_link_with_description() {
        let link = AdrLink::with_description(3, LinkKind::Amends, "Clarifies the API design");
        assert_eq!(link.target, 3);
        assert_eq!(link.kind, LinkKind::Amends);
        assert_eq!(link.description, Some("Clarifies the API design".into()));
    }

    // ========== Serialization Tests ==========

    #[test]
    fn test_adr_yaml_roundtrip() {
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;
        adr.date = time::Date::from_calendar_date(2024, Month::March, 15).unwrap();
        adr.add_link(AdrLink::new(2, LinkKind::Supersedes));

        let yaml = serde_yaml::to_string(&adr).unwrap();
        let parsed: Adr = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.number, adr.number);
        assert_eq!(parsed.title, adr.title);
        assert_eq!(parsed.status, adr.status);
        assert_eq!(parsed.date, adr.date);
        assert_eq!(parsed.links.len(), 1);
    }

    #[test]
    fn test_status_yaml_serialization() {
        assert_eq!(
            serde_yaml::to_string(&AdrStatus::Accepted).unwrap().trim(),
            "accepted"
        );
        assert_eq!(
            serde_yaml::to_string(&AdrStatus::Proposed).unwrap().trim(),
            "proposed"
        );
    }

    #[test]
    fn test_link_kind_yaml_serialization() {
        assert_eq!(
            serde_yaml::to_string(&LinkKind::Supersedes).unwrap().trim(),
            "supersedes"
        );
        assert_eq!(
            serde_yaml::to_string(&LinkKind::SupersededBy)
                .unwrap()
                .trim(),
            "supersededby"
        );
    }
}
