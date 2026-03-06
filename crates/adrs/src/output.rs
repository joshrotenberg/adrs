//! Output formatting for CLI commands.
//!
//! Provides structured output in JSON format for scripting and CI integration.
//! See Phase 1 of the CLI UX plan for design decisions.

use clap::ValueEnum;
use serde::Serialize;
use std::io::Write;

/// Output format for CLI commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text (default)
    #[default]
    Plain,
    /// JSON for scripting and CI
    Json,
}

/// Print a serializable value in the specified format.
///
/// For `Plain` format, this function does nothing - the caller handles plain output.
/// For `Json` format, outputs pretty-printed JSON to stdout.
pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{}", json);
    Ok(())
}

/// Print an error in the specified format.
///
/// For `Plain` format, outputs human-readable error to stderr.
/// For `Json` format, outputs structured JSON error to stderr.
#[allow(dead_code)]
pub fn print_error(error: &anyhow::Error, format: OutputFormat) {
    match format {
        OutputFormat::Plain => {
            eprintln!("error: {}", error);
            // Show cause chain
            let mut cause = error.source();
            while let Some(c) = cause {
                eprintln!("  caused by: {}", c);
                cause = std::error::Error::source(c);
            }
        }
        OutputFormat::Json => {
            let output = JsonError::from_anyhow(error);
            // Use stderr for errors, even in JSON mode
            if let Ok(json) = serde_json::to_string_pretty(&output) {
                let _ = writeln!(std::io::stderr(), "{}", json);
            }
        }
    }
}

/// Structured JSON error output.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct JsonError {
    pub error: ErrorDetails,
}

/// Error details for JSON output.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Error cause chain (optional)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub causes: Vec<String>,
}

#[allow(dead_code)]
impl JsonError {
    /// Create a JSON error from an anyhow error.
    pub fn from_anyhow(error: &anyhow::Error) -> Self {
        let mut causes = Vec::new();
        let mut cause = error.source();
        while let Some(c) = cause {
            causes.push(c.to_string());
            cause = std::error::Error::source(c);
        }

        Self {
            error: ErrorDetails {
                code: error_code_from_message(&error.to_string()),
                message: error.to_string(),
                context: None,
                causes,
            },
        }
    }

    /// Create a JSON error with a specific code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetails {
                code: code.into(),
                message: message.into(),
                context: None,
                causes: Vec::new(),
            },
        }
    }

    /// Add context to the error.
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.error.context = Some(context);
        self
    }
}

/// Derive an error code from the error message.
///
/// This is a heuristic - ideally errors would carry their own codes.
#[allow(dead_code)]
fn error_code_from_message(msg: &str) -> String {
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("not found") {
        if msg_lower.contains("adr") && !msg_lower.contains("repository") {
            return "ADR_NOT_FOUND".to_string();
        }
        if msg_lower.contains("repository") {
            return "REPO_NOT_FOUND".to_string();
        }
        return "NOT_FOUND".to_string();
    }

    if msg_lower.contains("config") || msg_lower.contains("configuration") {
        return "CONFIG_ERROR".to_string();
    }

    if msg_lower.contains("parse") || msg_lower.contains("invalid") {
        return "PARSE_ERROR".to_string();
    }

    if msg_lower.contains("permission") || msg_lower.contains("access denied") {
        return "PERMISSION_ERROR".to_string();
    }

    "ERROR".to_string()
}

// ============================================================================
// Output structs for each command
// ============================================================================

/// JSON output for `adrs list` command.
#[derive(Debug, Serialize)]
pub struct ListOutput {
    pub adrs: Vec<ListAdr>,
}

/// ADR entry in list output.
#[derive(Debug, Serialize)]
pub struct ListAdr {
    pub number: u32,
    pub title: String,
    pub status: String,
    pub date: String,
    pub path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deciders: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<ListAdrLink>,
}

/// Link in list output.
#[derive(Debug, Serialize)]
pub struct ListAdrLink {
    pub kind: String,
    pub target: u32,
}

/// JSON output for `adrs search` command.
#[derive(Debug, Serialize)]
pub struct SearchOutput {
    pub query: String,
    pub matches: Vec<SearchMatchOutput>,
    pub total: usize,
}

/// Search match in output.
#[derive(Debug, Serialize)]
pub struct SearchMatchOutput {
    pub number: u32,
    pub title: String,
    pub status: String,
    pub path: String,
    pub snippets: Vec<SearchSnippet>,
}

/// Search snippet showing where match was found.
#[derive(Debug, Serialize)]
pub struct SearchSnippet {
    pub section: String,
    pub text: String,
}

/// JSON output for `adrs config show` command.
#[derive(Debug, Serialize)]
pub struct ConfigOutput {
    pub root: String,
    pub source: String,
    pub config: ConfigValues,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<ConfigLayer>,
}

/// Configuration values.
#[derive(Debug, Serialize)]
pub struct ConfigValues {
    pub adr_dir: String,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_variant: Option<String>,
}

/// Configuration layer info (for verbose mode).
#[derive(Debug, Serialize)]
pub struct ConfigLayer {
    pub source: String,
    pub priority: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// JSON output for `adrs doctor` command.
#[derive(Debug, Serialize)]
pub struct DoctorOutput {
    pub healthy: bool,
    pub issues: Vec<DoctorIssue>,
    pub summary: DoctorSummary,
}

/// Issue found by doctor.
#[derive(Debug, Serialize)]
pub struct DoctorIssue {
    pub severity: String,
    pub rule_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adr_number: Option<u32>,
}

/// Summary of doctor findings.
#[derive(Debug, Serialize)]
pub struct DoctorSummary {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_from_message() {
        assert_eq!(error_code_from_message("ADR not found"), "ADR_NOT_FOUND");
        assert_eq!(
            error_code_from_message("Repository not found"),
            "REPO_NOT_FOUND"
        );
        assert_eq!(
            error_code_from_message("Config parse error"),
            "CONFIG_ERROR"
        );
        assert_eq!(error_code_from_message("Invalid format"), "PARSE_ERROR");
        assert_eq!(error_code_from_message("Something else"), "ERROR");
    }

    #[test]
    fn test_json_error_serialization() {
        let error = JsonError::new("TEST_ERROR", "Test message");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("TEST_ERROR"));
        assert!(json.contains("Test message"));
    }

    #[test]
    fn test_list_output_serialization() {
        let output = ListOutput {
            adrs: vec![ListAdr {
                number: 1,
                title: "Test".to_string(),
                status: "accepted".to_string(),
                date: "2024-01-15".to_string(),
                path: "doc/adr/0001-test.md".to_string(),
                tags: vec![],
                deciders: vec![],
                links: vec![],
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"number\":1"));
        assert!(json.contains("\"title\":\"Test\""));
        // Empty arrays should be omitted
        assert!(!json.contains("\"tags\""));
    }

    #[test]
    fn test_doctor_output_serialization() {
        let output = DoctorOutput {
            healthy: true,
            issues: vec![],
            summary: DoctorSummary {
                errors: 0,
                warnings: 0,
                info: 0,
            },
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"healthy\":true"));
    }
}
