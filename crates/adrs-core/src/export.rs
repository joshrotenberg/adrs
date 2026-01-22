//! JSON-ADR export functionality.
//!
//! Provides types and functions for exporting ADRs to the JSON-ADR format,
//! a machine-readable interchange format for Architecture Decision Records.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

use crate::{
    Adr, AdrLink, AdrStatus, Config, LinkKind, Parser, Repository, Result, TemplateEngine,
};

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

    /// Forces and concerns influencing the decision.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub decision_drivers: Vec<String>,

    /// Alternatives that were evaluated.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub considered_options: Vec<ConsideredOption>,

    /// The decision that was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,

    /// Outcomes and implications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consequences: Option<String>,

    /// How to validate the decision was implemented correctly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation: Option<String>,

    /// Relationships to other ADRs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<JsonAdrLink>,

    /// Custom sections not covered by standard fields.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub custom_sections: std::collections::HashMap<String, String>,

    /// Relative path to the source file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A considered option with pros and cons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsideredOption {
    /// Name of the option.
    pub name: String,

    /// Description of the option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Arguments in favor of this option.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub pros: Vec<String>,

    /// Arguments against this option.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cons: Vec<String>,
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
            decision_drivers: Vec::new(),   // Not yet in Adr type
            considered_options: Vec::new(), // Not yet in Adr type
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
            confirmation: None, // Not yet in Adr type
            links: adr.links.iter().map(JsonAdrLink::from).collect(),
            custom_sections: std::collections::HashMap::new(),
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

/// Single ADR wrapper for JSON-ADR format (used for single ADR import/export).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAdrSingle {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// JSON-ADR version.
    pub version: String,

    /// The ADR.
    pub adr: JsonAdr,
}

/// Options for importing ADRs.
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Overwrite existing files.
    pub overwrite: bool,

    /// Renumber ADRs starting from the next available number.
    pub renumber: bool,

    /// Use next-gen mode (YAML frontmatter).
    pub ng_mode: bool,
}

/// Result of an import operation.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Number of ADRs successfully imported.
    pub imported: usize,

    /// Number of ADRs skipped (already exist).
    pub skipped: usize,

    /// Paths of imported files.
    pub files: Vec<std::path::PathBuf>,

    /// Warnings encountered during import.
    pub warnings: Vec<String>,
}

/// Export all ADRs from a directory to JSON-ADR format.
///
/// This function scans a directory for markdown files that look like ADRs
/// (files matching `NNNN-*.md` pattern) and parses them. Unlike `export_repository`,
/// this does not require an initialized adrs repository.
pub fn export_directory(dir: &Path) -> Result<JsonAdrBulkExport> {
    let parser = Parser::new();
    let mut adrs = Vec::new();

    // Scan for markdown files
    if dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.is_file()
                    && path.extension().is_some_and(|ext| ext == "md")
                    && path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                        // Match NNNN-*.md pattern (adr-tools style)
                        n.len() > 5 && n[..4].chars().all(|c| c.is_ascii_digit())
                    })
            })
            .collect();

        // Sort by filename for consistent ordering
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            match parser.parse_file(&path) {
                Ok(adr) => adrs.push(JsonAdr::from(&adr)),
                Err(e) => {
                    // Log warning but continue with other files
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    let adr_dir = dir.display().to_string();
    Ok(JsonAdrBulkExport::new(adrs).with_repository(None, adr_dir))
}

/// Convert a JsonAdr back to an Adr for rendering.
fn json_adr_to_adr(json_adr: &JsonAdr) -> Result<Adr> {
    let date = time::Date::parse(
        &json_adr.date,
        &time::format_description::well_known::Iso8601::DATE,
    )
    .unwrap_or_else(|_| crate::parse::today());

    let status = json_adr.status.parse::<AdrStatus>().unwrap_or_default();

    let links: Vec<AdrLink> = json_adr
        .links
        .iter()
        .map(|l| AdrLink {
            target: l.target,
            kind: string_to_link_kind(&l.link_type),
            description: l.description.clone(),
        })
        .collect();

    Ok(Adr {
        number: json_adr.number,
        title: json_adr.title.clone(),
        date,
        status,
        links,
        decision_makers: json_adr.deciders.clone(),
        consulted: json_adr.consulted.clone(),
        informed: json_adr.informed.clone(),
        context: json_adr.context.clone().unwrap_or_default(),
        decision: json_adr.decision.clone().unwrap_or_default(),
        consequences: json_adr.consequences.clone().unwrap_or_default(),
        path: None,
    })
}

fn string_to_link_kind(s: &str) -> LinkKind {
    match s.to_lowercase().as_str() {
        "supersedes" => LinkKind::Supersedes,
        "superseded-by" => LinkKind::SupersededBy,
        "amends" => LinkKind::Amends,
        "amended-by" => LinkKind::AmendedBy,
        "relates-to" => LinkKind::RelatesTo,
        other => LinkKind::Custom(other.to_string()),
    }
}

/// Import ADRs from a JSON-ADR bulk export into a directory.
///
/// This creates markdown files from the JSON-ADR data. It can be used
/// to populate a new ADR directory or migrate ADRs between projects.
pub fn import_to_directory(
    json_data: &str,
    dir: &Path,
    options: &ImportOptions,
) -> Result<ImportResult> {
    // Parse the JSON - try bulk format first, then single
    let json_adrs: Vec<JsonAdr> =
        if let Ok(bulk) = serde_json::from_str::<JsonAdrBulkExport>(json_data) {
            bulk.adrs
        } else if let Ok(single) = serde_json::from_str::<JsonAdrSingle>(json_data) {
            vec![single.adr]
        } else if let Ok(adr) = serde_json::from_str::<JsonAdr>(json_data) {
            vec![adr]
        } else {
            return Err(crate::Error::InvalidFormat {
                path: PathBuf::new(),
                reason: "Invalid JSON-ADR format".to_string(),
            });
        };

    // Ensure directory exists
    std::fs::create_dir_all(dir)?;

    // If renumbering, find the next available number
    let mut next_number = if options.renumber {
        find_next_number(dir)?
    } else {
        0
    };

    let mut result = ImportResult {
        imported: 0,
        skipped: 0,
        files: Vec::new(),
        warnings: Vec::new(),
    };

    // Create config for template rendering
    let config = Config {
        adr_dir: dir.to_path_buf(),
        mode: if options.ng_mode {
            crate::ConfigMode::NextGen
        } else {
            crate::ConfigMode::default()
        },
        ..Default::default()
    };

    let engine = TemplateEngine::new();

    for json_adr in json_adrs {
        let mut adr = json_adr_to_adr(&json_adr)?;

        // Renumber if requested
        if options.renumber {
            adr.number = next_number;
            next_number += 1;
        }

        let filename = adr.filename();
        let filepath = dir.join(&filename);

        // Check if file exists
        if filepath.exists() && !options.overwrite {
            result.skipped += 1;
            result.warnings.push(format!(
                "Skipped {}: file already exists (use --overwrite to replace)",
                filename
            ));
            continue;
        }

        // Render the ADR to markdown
        let content = engine.render(&adr, &config)?;

        // Write the file
        std::fs::write(&filepath, content)?;

        result.imported += 1;
        result.files.push(filepath);
    }

    Ok(result)
}

/// Find the next available ADR number in a directory.
fn find_next_number(dir: &Path) -> Result<u32> {
    let mut max_number = 0u32;

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)?.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && name.len() > 4
                && name.ends_with(".md")
                && let Ok(num) = name[..4].parse::<u32>()
            {
                max_number = max_number.max(num);
            }
        }
    }

    Ok(max_number + 1)
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
            decision_drivers: vec![],
            considered_options: vec![],
            decision: None,
            consequences: None,
            confirmation: None,
            links: vec![],
            custom_sections: std::collections::HashMap::new(),
            path: None,
        };

        let json = serde_json::to_string(&adr).unwrap();
        assert!(json.contains("\"number\":1"));
        assert!(json.contains("\"title\":\"Test\""));
        // Empty vecs should be skipped
        assert!(!json.contains("\"deciders\""));
        assert!(!json.contains("\"decision_drivers\""));
        assert!(!json.contains("\"considered_options\""));
        // Empty custom_sections should be skipped
        assert!(!json.contains("\"custom_sections\""));
    }

    #[test]
    fn test_custom_sections() {
        let mut adr = JsonAdr {
            number: 1,
            title: "Test".to_string(),
            status: "Accepted".to_string(),
            date: "2024-01-15".to_string(),
            deciders: vec![],
            consulted: vec![],
            informed: vec![],
            tags: vec![],
            context: None,
            decision_drivers: vec![],
            considered_options: vec![],
            decision: None,
            consequences: None,
            confirmation: None,
            links: vec![],
            custom_sections: std::collections::HashMap::new(),
            path: None,
        };

        adr.custom_sections.insert(
            "Alternatives Considered".to_string(),
            "We also looked at MySQL and SQLite.".to_string(),
        );
        adr.custom_sections.insert(
            "Security Review".to_string(),
            "Approved by security team on 2024-01-10.".to_string(),
        );

        let json = serde_json::to_string_pretty(&adr).unwrap();
        assert!(json.contains("\"custom_sections\""));
        assert!(json.contains("Alternatives Considered"));
        assert!(json.contains("Security Review"));
    }

    #[test]
    fn test_decision_drivers_and_options() {
        let adr = JsonAdr {
            number: 1,
            title: "Choose Database".to_string(),
            status: "Accepted".to_string(),
            date: "2024-01-15".to_string(),
            deciders: vec!["Alice".to_string()],
            consulted: vec![],
            informed: vec![],
            tags: vec![],
            context: Some("We need a database for user data".to_string()),
            decision_drivers: vec![
                "Performance requirements".to_string(),
                "Team expertise".to_string(),
                "Cost constraints".to_string(),
            ],
            considered_options: vec![
                ConsideredOption {
                    name: "PostgreSQL".to_string(),
                    description: Some("Open source relational database".to_string()),
                    pros: vec!["Mature".to_string(), "Feature-rich".to_string()],
                    cons: vec!["Complex setup".to_string()],
                },
                ConsideredOption {
                    name: "SQLite".to_string(),
                    description: None,
                    pros: vec!["Simple".to_string()],
                    cons: vec!["Not suitable for high concurrency".to_string()],
                },
            ],
            decision: Some("Use PostgreSQL".to_string()),
            consequences: Some("Need to set up replication".to_string()),
            confirmation: Some("Run integration tests with production-like data".to_string()),
            links: vec![],
            custom_sections: std::collections::HashMap::new(),
            path: None,
        };

        let json = serde_json::to_string_pretty(&adr).unwrap();

        // Check decision drivers
        assert!(json.contains("\"decision_drivers\""));
        assert!(json.contains("Performance requirements"));
        assert!(json.contains("Team expertise"));

        // Check considered options
        assert!(json.contains("\"considered_options\""));
        assert!(json.contains("PostgreSQL"));
        assert!(json.contains("SQLite"));
        assert!(json.contains("\"pros\""));
        assert!(json.contains("\"cons\""));
        assert!(json.contains("Mature"));
        assert!(json.contains("Complex setup"));

        // Check confirmation
        assert!(json.contains("\"confirmation\""));
        assert!(json.contains("integration tests"));
    }

    #[test]
    fn test_considered_option_minimal() {
        let option = ConsideredOption {
            name: "Option A".to_string(),
            description: None,
            pros: vec![],
            cons: vec![],
        };

        let json = serde_json::to_string(&option).unwrap();
        assert!(json.contains("\"name\":\"Option A\""));
        // Empty fields should be skipped
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"pros\""));
        assert!(!json.contains("\"cons\""));
    }
}
