//! JSON-ADR export functionality.
//!
//! Provides types and functions for exporting ADRs to the JSON-ADR format,
//! a machine-readable interchange format for Architecture Decision Records.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{Adr, AdrLink, LinkKind, Repository, Result};

/// JSON-ADR schema version.
pub const JSON_ADR_VERSION: &str = "1.0.0";

/// JSON-ADR schema URL.
pub const JSON_ADR_SCHEMA: &str =
    "https://raw.githubusercontent.com/joshrotenberg/adrs/main/schema/json-adr/v1.json";

/// A single ADR in JSON-ADR format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAdr {
    /// Unique identifier for the ADR.
    pub number: u32,

    /// Title of the decision.
    pub title: String,

    /// Current status.
    pub status: String,

    /// Date when the decision was made (ISO 8601).
    pub date: String,

    /// People who made the decision.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub deciders: Vec<String>,

    /// People whose opinions were sought.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub consulted: Vec<String>,

    /// People informed after the decision.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub informed: Vec<String>,

    /// Categorization labels.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,

    /// Background and problem statement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// The decision that was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,

    /// Outcomes and implications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consequences: Option<String>,

    /// Relationships to other ADRs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<JsonAdrLink>,

    /// Relative path to the source file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A link between ADRs in JSON-ADR format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAdrLink {
    /// Link type.
    #[serde(rename = "type")]
    pub link_type: String,

    /// ADR number being linked to.
    pub target: u32,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Tool metadata for bulk exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name.
    pub name: String,

    /// Tool version.
    pub version: String,
}

/// Repository metadata for bulk exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository/project name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// ADR directory path.
    pub adr_directory: String,
}

/// Bulk export of multiple ADRs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAdrBulkExport {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// JSON-ADR version.
    pub version: String,

    /// When the export was generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,

    /// Tool that generated the export.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<ToolInfo>,

    /// Repository metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryInfo>,

    /// The ADRs.
    pub adrs: Vec<JsonAdr>,
}

impl JsonAdrBulkExport {
    /// Create a new bulk export with default metadata.
    pub fn new(adrs: Vec<JsonAdr>) -> Self {
        Self {
            schema: Some(JSON_ADR_SCHEMA.to_string()),
            version: JSON_ADR_VERSION.to_string(),
            generated_at: Some(
                OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            ),
            tool: Some(ToolInfo {
                name: "adrs".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            repository: None,
            adrs,
        }
    }

    /// Set repository metadata.
    pub fn with_repository(mut self, name: Option<String>, adr_directory: String) -> Self {
        self.repository = Some(RepositoryInfo {
            name,
            adr_directory,
        });
        self
    }
}

impl From<&Adr> for JsonAdr {
    fn from(adr: &Adr) -> Self {
        Self {
            number: adr.number,
            title: adr.title.clone(),
            status: adr.status.to_string(),
            date: adr
                .date
                .format(&time::format_description::well_known::Iso8601::DATE)
                .unwrap_or_default(),
            deciders: adr.decision_makers.clone(),
            consulted: adr.consulted.clone(),
            informed: adr.informed.clone(),
            tags: Vec::new(), // Tags not yet implemented in Adr
            context: if adr.context.is_empty() {
                None
            } else {
                Some(adr.context.clone())
            },
            decision: if adr.decision.is_empty() {
                None
            } else {
                Some(adr.decision.clone())
            },
            consequences: if adr.consequences.is_empty() {
                None
            } else {
                Some(adr.consequences.clone())
            },
            links: adr.links.iter().map(JsonAdrLink::from).collect(),
            path: adr.path.as_ref().map(|p| p.display().to_string()),
        }
    }
}

impl From<&AdrLink> for JsonAdrLink {
    fn from(link: &AdrLink) -> Self {
        Self {
            link_type: link_kind_to_string(&link.kind),
            target: link.target,
            description: link.description.clone(),
        }
    }
}

fn link_kind_to_string(kind: &LinkKind) -> String {
    match kind {
        LinkKind::Supersedes => "supersedes".to_string(),
        LinkKind::SupersededBy => "superseded-by".to_string(),
        LinkKind::Amends => "amends".to_string(),
        LinkKind::AmendedBy => "amended-by".to_string(),
        LinkKind::RelatesTo => "relates-to".to_string(),
        LinkKind::Custom(s) => s.clone(),
    }
}

/// Export all ADRs from a repository to JSON-ADR format.
pub fn export_repository(repo: &Repository) -> Result<JsonAdrBulkExport> {
    let adrs = repo.list()?;
    let json_adrs: Vec<JsonAdr> = adrs.iter().map(JsonAdr::from).collect();

    let adr_dir = repo.config().adr_dir.display().to_string();

    Ok(JsonAdrBulkExport::new(json_adrs).with_repository(None, adr_dir))
}

/// Export a single ADR to JSON-ADR format.
pub fn export_adr(adr: &Adr) -> JsonAdr {
    JsonAdr::from(adr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AdrStatus;
    use time::{Date, Month};

    #[test]
    fn test_json_adr_from_adr() {
        let mut adr = Adr::new(1, "Test Decision");
        adr.status = AdrStatus::Accepted;
        adr.date = Date::from_calendar_date(2024, Month::January, 15).unwrap();
        adr.context = "Some context".to_string();
        adr.decision = "We decided X".to_string();
        adr.consequences = "This means Y".to_string();
        adr.decision_makers = vec!["Alice".to_string()];
        adr.consulted = vec!["Bob".to_string()];

        let json_adr = JsonAdr::from(&adr);

        assert_eq!(json_adr.number, 1);
        assert_eq!(json_adr.title, "Test Decision");
        assert_eq!(json_adr.status, "Accepted");
        assert_eq!(json_adr.date, "2024-01-15");
        assert_eq!(json_adr.deciders, vec!["Alice"]);
        assert_eq!(json_adr.consulted, vec!["Bob"]);
    }

    #[test]
    fn test_json_adr_link_from_adr_link() {
        let link = AdrLink {
            target: 2,
            kind: LinkKind::Supersedes,
            description: Some("Replaces old approach".to_string()),
        };

        let json_link = JsonAdrLink::from(&link);

        assert_eq!(json_link.link_type, "supersedes");
        assert_eq!(json_link.target, 2);
        assert_eq!(
            json_link.description,
            Some("Replaces old approach".to_string())
        );
    }

    #[test]
    fn test_bulk_export_metadata() {
        let export = JsonAdrBulkExport::new(vec![]);

        assert_eq!(export.version, JSON_ADR_VERSION);
        assert!(export.schema.is_some());
        assert!(export.generated_at.is_some());
        assert!(export.tool.is_some());

        let tool = export.tool.unwrap();
        assert_eq!(tool.name, "adrs");
    }

    #[test]
    fn test_bulk_export_with_repository() {
        let export = JsonAdrBulkExport::new(vec![])
            .with_repository(Some("my-project".to_string()), "doc/adr".to_string());

        let repo = export.repository.unwrap();
        assert_eq!(repo.name, Some("my-project".to_string()));
        assert_eq!(repo.adr_directory, "doc/adr");
    }

    #[test]
    fn test_link_kind_to_string() {
        assert_eq!(link_kind_to_string(&LinkKind::Supersedes), "supersedes");
        assert_eq!(
            link_kind_to_string(&LinkKind::SupersededBy),
            "superseded-by"
        );
        assert_eq!(link_kind_to_string(&LinkKind::Amends), "amends");
        assert_eq!(link_kind_to_string(&LinkKind::AmendedBy), "amended-by");
        assert_eq!(link_kind_to_string(&LinkKind::RelatesTo), "relates-to");
        assert_eq!(
            link_kind_to_string(&LinkKind::Custom("extends".to_string())),
            "extends"
        );
    }

    #[test]
    fn test_json_serialization() {
        let adr = JsonAdr {
            number: 1,
            title: "Test".to_string(),
            status: "Accepted".to_string(),
            date: "2024-01-15".to_string(),
            deciders: vec![],
            consulted: vec![],
            informed: vec![],
            tags: vec![],
            context: None,
            decision: None,
            consequences: None,
            links: vec![],
            path: None,
        };

        let json = serde_json::to_string(&adr).unwrap();
        assert!(json.contains("\"number\":1"));
        assert!(json.contains("\"title\":\"Test\""));
        // Empty vecs should be skipped
        assert!(!json.contains("\"deciders\""));
    }
}
