//! MCP (Model Context Protocol) server for ADR integration.
//!
//! This module provides an MCP server that allows AI agents to interact with ADRs.
//! Enable with the `mcp` feature flag.

use adrs_core::{
    AdrStatus, IssueSeverity, LinkKind, Repository, TemplateFormat, TemplateVariant, check_all,
    export_repository,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use time::Date;
use tower_mcp::{CallToolResult, McpRouter, ToolBuilder};

/// Shared state for ADR MCP tools.
#[derive(Debug, Clone)]
struct AdrState {
    root: PathBuf,
}

impl AdrState {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn open_repo(&self) -> Result<Repository, String> {
        Repository::open(&self.root).map_err(|e| e.to_string())
    }
}

// Tool parameter types

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListAdrsParams {
    /// Filter by status (e.g., "proposed", "accepted", "deprecated", "superseded")
    #[schemars(description = "Filter ADRs by status (optional)")]
    pub status: Option<String>,

    /// Filter by tag (requires NextGen mode)
    #[schemars(description = "Filter ADRs by tag (optional)")]
    pub tag: Option<String>,

    /// Filter ADRs created on or after this date (YYYY-MM-DD, inclusive)
    #[schemars(
        description = "Filter ADRs created on or after this date in YYYY-MM-DD format (optional)"
    )]
    pub since: Option<String>,

    /// Filter ADRs created on or before this date (YYYY-MM-DD, inclusive)
    #[schemars(
        description = "Filter ADRs created on or before this date in YYYY-MM-DD format (optional)"
    )]
    pub until: Option<String>,

    /// Filter by decider name (case-insensitive substring match against decision_makers)
    #[schemars(
        description = "Filter ADRs by decider name -- case-insensitive substring match (optional)"
    )]
    pub decider: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAdrParams {
    /// ADR number to retrieve
    #[schemars(description = "The ADR number (e.g., 1, 2, 3)")]
    pub number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchAdrsParams {
    /// Search query
    #[schemars(description = "Text to search for in ADR titles and content")]
    pub query: String,

    /// Search titles only
    #[schemars(description = "If true, only search in ADR titles (optional)")]
    pub title_only: Option<bool>,

    /// Filter results by ADR status (optional)
    #[schemars(description = "Filter search results by status (optional)")]
    pub status: Option<String>,

    /// If true, perform a case-sensitive search (default: false)
    #[schemars(description = "If true, perform a case-sensitive search (default: false)")]
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportAdrsParams {
    /// If provided, only export ADRs with these numbers
    #[schemars(description = "List of ADR numbers to export (optional, exports all if omitted)")]
    pub numbers: Option<Vec<u32>>,

    /// If true, omit content fields (context, decision, consequences) from output
    #[schemars(description = "If true, export metadata only without content sections (optional)")]
    pub metadata_only: Option<bool>,
}

// Write operation parameter types

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateAdrParams {
    /// Title of the ADR
    #[schemars(description = "Title describing the architectural decision")]
    pub title: String,

    /// Context/background for the decision
    #[schemars(
        description = "Context explaining why this decision is needed (optional, can be filled in later)"
    )]
    pub context: Option<String>,

    /// The decision that was made
    #[schemars(description = "The actual decision statement (optional, can be filled in later)")]
    pub decision: Option<String>,

    /// Consequences of the decision
    #[schemars(
        description = "Expected consequences of this decision (optional, can be filled in later)"
    )]
    pub consequences: Option<String>,

    /// ADR number this supersedes (if any)
    #[schemars(description = "If this ADR supersedes another, provide its number (optional)")]
    pub supersedes: Option<u32>,

    /// Template format: "nygard" (default) or "madr"
    #[schemars(
        description = "Template format: 'nygard' (default) or 'madr' for MADR 4.0.0 format"
    )]
    pub format: Option<String>,

    /// Template variant: "full" (default), "minimal", or "bare"
    #[schemars(description = "Template variant: 'full' (default), 'minimal', or 'bare'")]
    pub variant: Option<String>,

    /// Decision makers for MADR format
    #[schemars(
        description = "People who made the decision (optional, most useful with MADR format)"
    )]
    pub decision_makers: Option<Vec<String>>,

    /// Consulted people for MADR format
    #[schemars(
        description = "People consulted for input (optional, most useful with MADR format)"
    )]
    pub consulted: Option<Vec<String>>,

    /// Informed people for MADR format
    #[schemars(description = "People kept informed (optional, most useful with MADR format)")]
    pub informed: Option<Vec<String>>,

    /// Tags for categorization (applied in NextGen mode; warns in compatible mode)
    #[schemars(
        description = "Tags for categorization. Applied at creation in NextGen mode. In compatible mode, a warning is returned instead of an error."
    )]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateStatusParams {
    /// ADR number to update
    #[schemars(description = "The ADR number to update")]
    pub number: u32,

    /// New status
    #[schemars(
        description = "New status: proposed, accepted, deprecated, superseded, or rejected"
    )]
    pub status: String,

    /// For superseded status, the ADR that supersedes this one
    #[schemars(description = "If status is 'superseded', the ADR number that supersedes this one")]
    pub superseded_by: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LinkAdrsParams {
    /// Source ADR number
    #[schemars(description = "The ADR number that is the source of the link")]
    pub source: u32,

    /// Target ADR number
    #[schemars(description = "The ADR number that is the target of the link")]
    pub target: u32,

    /// Link type
    #[schemars(description = "Link type: Supersedes, Amends, or 'Relates to'")]
    pub link_type: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateContentParams {
    /// ADR number to update
    #[schemars(description = "The ADR number to update")]
    pub number: u32,

    /// Context/background for the decision
    #[schemars(description = "New context section content (optional, omit to keep existing)")]
    pub context: Option<String>,

    /// The decision that was made
    #[schemars(description = "New decision section content (optional, omit to keep existing)")]
    pub decision: Option<String>,

    /// Consequences of the decision
    #[schemars(description = "New consequences section content (optional, omit to keep existing)")]
    pub consequences: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTagsParams {
    /// ADR number to update
    #[schemars(description = "The ADR number to update")]
    pub number: u32,

    /// Tags to add
    #[schemars(description = "Tags to add to the ADR (requires NextGen mode)")]
    pub tags: Vec<String>,

    /// Replace existing tags instead of appending
    #[schemars(description = "If true, replace all existing tags; if false, append to existing")]
    pub replace: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetRelatedParams {
    /// ADR number
    #[schemars(description = "The ADR number to get related ADRs for")]
    pub number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateAdrParams {
    /// ADR number to validate
    #[schemars(description = "The ADR number to validate")]
    pub number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAdrSectionsParams {
    /// ADR number to retrieve
    #[schemars(description = "The ADR number (e.g., 1, 2, 3)")]
    pub number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompareAdrsParams {
    /// First ADR number (source)
    #[schemars(description = "The first ADR number to compare (source)")]
    pub source: u32,

    /// Second ADR number (target)
    #[schemars(description = "The second ADR number to compare (target)")]
    pub target: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BulkUpdateStatusParams {
    /// ADR numbers to update
    #[schemars(description = "Array of ADR numbers to update")]
    pub numbers: Vec<u32>,

    /// New status for all ADRs
    #[schemars(
        description = "New status: proposed, accepted, deprecated, superseded, or rejected"
    )]
    pub status: String,

    /// For superseded status, the ADR that supersedes these
    #[schemars(description = "If status is 'superseded', the ADR number that supersedes these")]
    pub superseded_by: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SuggestTagsParams {
    /// ADR number to analyze
    #[schemars(description = "The ADR number to analyze for tag suggestions")]
    pub number: u32,

    /// Maximum number of tags to suggest
    #[schemars(description = "Maximum number of tags to suggest (default: 5)")]
    pub max_tags: Option<usize>,
}

// Response types

#[derive(Debug, Serialize, Deserialize)]
struct AdrSummary {
    number: u32,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdrDetail {
    number: u32,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
    content: String,
    links: Vec<LinkInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LinkInfo {
    kind: String,
    target: u32,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    number: u32,
    title: String,
    valid: bool,
    issues: Vec<ValidationIssue>,
    sections: SectionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationIssue {
    severity: String,
    message: String,
    section: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SectionStatus {
    context: SectionInfo,
    decision: SectionInfo,
    consequences: SectionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct SectionInfo {
    present: bool,
    empty: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdrSections {
    number: u32,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
    context: String,
    decision: String,
    consequences: String,
    links: Vec<LinkInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompareResult {
    source: AdrBrief,
    target: AdrBrief,
    differences: Differences,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdrBrief {
    number: u32,
    title: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Differences {
    title_changed: bool,
    status_changed: bool,
    context: SectionDiff,
    decision: SectionDiff,
    consequences: SectionDiff,
}

#[derive(Debug, Serialize, Deserialize)]
struct SectionDiff {
    changed: bool,
    source_empty: bool,
    target_empty: bool,
    source_length: usize,
    target_length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct BulkUpdateResult {
    updated: Vec<UpdatedAdr>,
    failed: Vec<FailedAdr>,
    summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdatedAdr {
    number: u32,
    old_status: String,
    new_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FailedAdr {
    number: u32,
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuggestTagsResult {
    number: u32,
    title: String,
    existing_tags: Vec<String>,
    suggested_tags: Vec<SuggestedTag>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuggestedTag {
    tag: String,
    confidence: f32,
    reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    number: u32,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
    matches: Vec<MatchSnippet>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MatchSnippet {
    section: String,
    snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoctorResult {
    healthy: bool,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    issues: Vec<DoctorIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoctorIssue {
    severity: String,
    rule_id: String,
    rule_name: String,
    message: String,
    path: Option<String>,
    line: Option<usize>,
    adr_number: Option<u32>,
}

// Macro to reduce boilerplate for tool registration.
macro_rules! adr_tool {
    // Read-only tool with params.
    (ro, $state:expr, $name:expr, $desc:expr, $param:ty, $method:ident) => {
        ToolBuilder::new($name)
            .description($desc)
            .read_only()
            .handler({
                let state = $state.clone();
                move |params: $param| {
                    let state = state.clone();
                    async move {
                        match state.$method(params) {
                            Ok(json) => Ok(CallToolResult::text(json)),
                            Err(e) => Ok(CallToolResult::error(format!("Error: {e}"))),
                        }
                    }
                }
            })
            .build()
    };
    // Write tool with params.
    (rw, $state:expr, $name:expr, $desc:expr, $param:ty, $method:ident) => {
        ToolBuilder::new($name)
            .description($desc)
            .handler({
                let state = $state.clone();
                move |params: $param| {
                    let state = state.clone();
                    async move {
                        match state.$method(params) {
                            Ok(json) => Ok(CallToolResult::text(json)),
                            Err(e) => Ok(CallToolResult::error(format!("Error: {e}"))),
                        }
                    }
                }
            })
            .build()
    };
    // Read-only tool without params.
    (ro, $state:expr, $name:expr, $desc:expr, $method:ident) => {
        ToolBuilder::new($name)
            .description($desc)
            .read_only()
            .no_params_handler({
                let state = $state.clone();
                move || {
                    let state = state.clone();
                    async move {
                        match state.$method() {
                            Ok(json) => Ok(CallToolResult::text(json)),
                            Err(e) => Ok(CallToolResult::error(format!("Error: {e}"))),
                        }
                    }
                }
            })
            .build()
    };
}

/// Build the MCP router with all ADR tools registered.
fn build_router(root: PathBuf) -> McpRouter {
    let state = Arc::new(AdrState::new(root));

    // Read-only tools
    let list_adrs = adr_tool!(
        ro,
        state,
        "list_adrs",
        "List all Architecture Decision Records. Returns summary information for each ADR including number, title, status, and date. Optionally filter by status or tag.",
        ListAdrsParams,
        list_adrs_impl
    );

    let get_adr = adr_tool!(
        ro,
        state,
        "get_adr",
        "Get the full content of an Architecture Decision Record by its number. Returns the complete ADR including title, status, content, and links.",
        GetAdrParams,
        get_adr_impl
    );

    let search_adrs = adr_tool!(
        ro,
        state,
        "search_adrs",
        "Search Architecture Decision Records for matching text. Searches both titles and content by default. Use title_only=true to search only titles.",
        SearchAdrsParams,
        search_adrs_impl
    );

    let get_repository_info = adr_tool!(
        ro,
        state,
        "get_repository_info",
        "Get information about the ADR repository including mode (compatible/nextgen), ADR count, and configuration.",
        get_repository_info_impl
    );

    let get_related_adrs = adr_tool!(
        ro,
        state,
        "get_related_adrs",
        "Get all ADRs that are linked to or from a specific ADR. Returns both incoming and outgoing links with their types.",
        GetRelatedParams,
        get_related_adrs_impl
    );

    let validate_adr = adr_tool!(
        ro,
        state,
        "validate_adr",
        "Validate a single ADR's structure and content. Checks for required sections (Context, Decision, Consequences), validates status, and reports any issues. Returns validation results with severity levels (error/warning).",
        ValidateAdrParams,
        validate_adr_impl
    );

    let get_adr_sections = adr_tool!(
        ro,
        state,
        "get_adr_sections",
        "Get an ADR with its content parsed into separate sections (context, decision, consequences). Returns structured data instead of raw markdown, making it easier to analyze specific sections independently.",
        GetAdrSectionsParams,
        get_adr_sections_impl
    );

    let compare_adrs = adr_tool!(
        ro,
        state,
        "compare_adrs",
        "Compare two ADRs and show the differences between them. Useful for understanding how decisions evolved, especially when one ADR supersedes another. Returns structural comparison of title, status, and content sections.",
        CompareAdrsParams,
        compare_adrs_impl
    );

    let suggest_tags = adr_tool!(
        ro,
        state,
        "suggest_tags",
        "Analyze an ADR's content and suggest relevant tags based on keywords and common architectural categories. Returns suggested tags with confidence scores and reasons. Requires NextGen mode for tags to be applied.",
        SuggestTagsParams,
        suggest_tags_impl
    );

    let run_doctor = adr_tool!(
        ro,
        state,
        "run_doctor",
        "Run health checks on the ADR repository. Returns broken links, parse errors, duplicate numbers, and other issues. Read-only; does not modify any files.",
        run_doctor_impl
    );

    let export_adrs = adr_tool!(
        ro,
        state,
        "export_adrs",
        "Export ADRs to JSON-ADR format (machine-readable interchange format). Optionally filter to specific ADR numbers and/or export metadata only without content sections.",
        ExportAdrsParams,
        export_adrs_impl
    );

    // Write tools
    let create_adr = adr_tool!(
        rw,
        state,
        "create_adr",
        "Create a new Architecture Decision Record. The ADR will use the repository's configured default status (proposed if not configured) and requires human review before acceptance. Returns the created ADR details including its number and file path.",
        CreateAdrParams,
        create_adr_impl
    );

    let update_status = adr_tool!(
        rw,
        state,
        "update_status",
        "Update the status of an existing ADR. Valid statuses: proposed, accepted, deprecated, superseded, rejected. For 'superseded', provide the superseded_by number. Note: Status changes should be reviewed by humans.",
        UpdateStatusParams,
        update_status_impl
    );

    let link_adrs = adr_tool!(
        rw,
        state,
        "link_adrs",
        "Create a bidirectional link between two ADRs. Link types: 'Supersedes', 'Amends', or 'Relates to'. The reverse link is automatically created on the target ADR.",
        LinkAdrsParams,
        link_adrs_impl
    );

    let update_content = adr_tool!(
        rw,
        state,
        "update_content",
        "Update the content sections (context, decision, consequences) of an existing ADR. Only provided fields are updated; omitted fields are preserved. Changes should be reviewed by humans.",
        UpdateContentParams,
        update_content_impl
    );

    let update_tags = adr_tool!(
        rw,
        state,
        "update_tags",
        "Add or replace tags on an ADR. Requires NextGen mode (YAML frontmatter). Use replace=true to replace all tags, or false/omit to append.",
        UpdateTagsParams,
        update_tags_impl
    );

    let bulk_update_status = adr_tool!(
        rw,
        state,
        "bulk_update_status",
        "Update the status of multiple ADRs in a single operation. Useful for batch accepting related ADRs or deprecating multiple outdated decisions. Returns detailed results for each ADR including any failures.",
        BulkUpdateStatusParams,
        bulk_update_status_impl
    );

    McpRouter::new()
        .server_info("adrs", env!("CARGO_PKG_VERSION"))
        .auto_instructions()
        // Read tools
        .tool(list_adrs)
        .tool(get_adr)
        .tool(search_adrs)
        .tool(get_repository_info)
        .tool(get_related_adrs)
        .tool(validate_adr)
        .tool(get_adr_sections)
        .tool(compare_adrs)
        .tool(suggest_tags)
        .tool(run_doctor)
        .tool(export_adrs)
        // Write tools
        .tool(create_adr)
        .tool(update_status)
        .tool(link_adrs)
        .tool(update_content)
        .tool(update_tags)
        .tool(bulk_update_status)
}

/// Check if a section's text contains the (already-normalized) query.
fn search_section_matches(text: &str, query_normalized: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        text.contains(query_normalized)
    } else {
        text.to_lowercase().contains(query_normalized)
    }
}

/// Extract a context snippet around the match location in text.
/// The query_normalized must already be lowercased if case_sensitive is false.
fn extract_search_snippet(text: &str, query_normalized: &str, case_sensitive: bool) -> String {
    let text_search = if case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    };

    if let Some(pos) = text_search.find(query_normalized) {
        let start = pos.saturating_sub(40);
        let end = (pos + query_normalized.len() + 40).min(text.len());

        // Expand to word boundaries
        let start = text[..start]
            .rfind(char::is_whitespace)
            .map(|p| p + 1)
            .unwrap_or(start);
        let end = text[end..]
            .find(char::is_whitespace)
            .map(|p| end + p)
            .unwrap_or(end);

        let mut snippet = text[start..end].to_string();

        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < text.len() {
            snippet = format!("{}...", snippet);
        }

        snippet.replace('\n', " ")
    } else {
        // Fallback: first 80 chars
        let preview: String = text.chars().take(80).collect();
        if text.len() > 80 {
            format!("{}...", preview)
        } else {
            preview
        }
    }
}

// Business logic implementations.

impl AdrState {
    fn list_adrs_impl(&self, params: ListAdrsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adrs = repo.list().map_err(|e| e.to_string())?;

        // Parse date filters
        let since_date: Option<Date> = match &params.since {
            Some(s) => Some(
                Date::parse(s, &time::format_description::well_known::Iso8601::DATE)
                    .map_err(|_| format!("Invalid date format: '{}'. Use YYYY-MM-DD.", s))?,
            ),
            None => None,
        };

        let until_date: Option<Date> = match &params.until {
            Some(s) => Some(
                Date::parse(s, &time::format_description::well_known::Iso8601::DATE)
                    .map_err(|_| format!("Invalid date format: '{}'. Use YYYY-MM-DD.", s))?,
            ),
            None => None,
        };

        let summaries: Vec<AdrSummary> = adrs
            .iter()
            .filter(|adr| {
                // Status filter (case-insensitive)
                if let Some(ref status) = params.status
                    && adr.status.to_string().to_lowercase() != status.to_lowercase()
                {
                    return false;
                }

                // Tag filter (case-insensitive)
                if let Some(ref tag) = params.tag {
                    let tag_lower = tag.to_lowercase();
                    if !adr.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                        return false;
                    }
                }

                // Since date filter (inclusive)
                if let Some(since) = since_date
                    && adr.date < since
                {
                    return false;
                }

                // Until date filter (inclusive)
                if let Some(until) = until_date
                    && adr.date > until
                {
                    return false;
                }

                // Decider filter (case-insensitive substring match)
                if let Some(ref decider) = params.decider {
                    let decider_lower = decider.to_lowercase();
                    if !adr
                        .decision_makers
                        .iter()
                        .any(|dm| dm.to_lowercase().contains(&decider_lower))
                    {
                        return false;
                    }
                }

                true
            })
            .map(|adr| AdrSummary {
                number: adr.number,
                title: adr.title.clone(),
                status: adr.status.to_string(),
                date: Some(adr.date.to_string()),
                tags: adr.tags.clone(),
            })
            .collect();

        serde_json::to_string_pretty(&summaries).map_err(|e| e.to_string())
    }

    fn get_adr_impl(&self, params: GetAdrParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adr = repo.get(params.number).map_err(|e| e.to_string())?;

        let content = repo.read_content(&adr).map_err(|e| e.to_string())?;

        let detail = AdrDetail {
            number: adr.number,
            title: adr.title.clone(),
            status: adr.status.to_string(),
            date: Some(adr.date.to_string()),
            tags: adr.tags.clone(),
            content,
            links: adr
                .links
                .iter()
                .map(|l| LinkInfo {
                    kind: l.kind.to_string(),
                    target: l.target,
                    description: l.description.clone(),
                })
                .collect(),
        };

        serde_json::to_string_pretty(&detail).map_err(|e| e.to_string())
    }

    fn search_adrs_impl(&self, params: SearchAdrsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adrs = repo.list().map_err(|e| e.to_string())?;

        let case_sensitive = params.case_sensitive.unwrap_or(false);
        let title_only = params.title_only.unwrap_or(false);

        // Normalize the query based on case sensitivity
        let query_normalized = if case_sensitive {
            params.query.clone()
        } else {
            params.query.to_lowercase()
        };

        let mut results: Vec<SearchResult> = Vec::new();

        for adr in &adrs {
            // Apply status filter (case-insensitive)
            if let Some(ref status) = params.status
                && adr.status.to_string().to_lowercase() != status.to_lowercase()
            {
                continue;
            }

            let mut snippets: Vec<MatchSnippet> = Vec::new();

            // Check title
            let title_text = if case_sensitive {
                adr.title.clone()
            } else {
                adr.title.to_lowercase()
            };
            if title_text.contains(&query_normalized) {
                snippets.push(MatchSnippet {
                    section: "Title".to_string(),
                    snippet: adr.title.clone(),
                });
            }

            if !title_only {
                // Check context
                if search_section_matches(&adr.context, &query_normalized, case_sensitive) {
                    snippets.push(MatchSnippet {
                        section: "Context".to_string(),
                        snippet: extract_search_snippet(
                            &adr.context,
                            &query_normalized,
                            case_sensitive,
                        ),
                    });
                }

                // Check decision
                if search_section_matches(&adr.decision, &query_normalized, case_sensitive) {
                    snippets.push(MatchSnippet {
                        section: "Decision".to_string(),
                        snippet: extract_search_snippet(
                            &adr.decision,
                            &query_normalized,
                            case_sensitive,
                        ),
                    });
                }

                // Check consequences
                if search_section_matches(&adr.consequences, &query_normalized, case_sensitive) {
                    snippets.push(MatchSnippet {
                        section: "Consequences".to_string(),
                        snippet: extract_search_snippet(
                            &adr.consequences,
                            &query_normalized,
                            case_sensitive,
                        ),
                    });
                }
            }

            if !snippets.is_empty() {
                results.push(SearchResult {
                    number: adr.number,
                    title: adr.title.clone(),
                    status: adr.status.to_string(),
                    date: Some(adr.date.to_string()),
                    tags: adr.tags.clone(),
                    matches: snippets,
                });
            }
        }

        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    }

    fn create_adr_impl(&self, params: CreateAdrParams) -> Result<String, String> {
        // Parse format/variant before opening repo so we fail fast on invalid input.
        let template_format = match &params.format {
            Some(f) => f
                .parse::<TemplateFormat>()
                .map_err(|_| format!("Invalid format '{}'. Use 'nygard' or 'madr'.", f))?,
            None => TemplateFormat::default(),
        };

        let template_variant = match &params.variant {
            Some(v) => v.parse::<TemplateVariant>().map_err(|_| {
                format!(
                    "Invalid variant '{}'. Use 'full', 'minimal', 'bare', or 'bare-minimal'.",
                    v
                )
            })?,
            None => TemplateVariant::default(),
        };

        let repo = self
            .open_repo()?
            .with_template_format(template_format)
            .with_template_variant(template_variant);

        let mut warnings: Vec<String> = Vec::new();

        let (mut adr, path) = if let Some(supersedes) = params.supersedes {
            repo.supersede(&params.title, supersedes)
                .map_err(|e| e.to_string())?
        } else {
            repo.new_adr(&params.title).map_err(|e| e.to_string())?
        };

        // Update content sections if provided.
        let mut content_updated = false;
        let mut body = adrs_core::BodySectionPatch::default();
        if let Some(context) = params.context {
            adr.context = context.clone();
            body.context = Some(context);
            content_updated = true;
        }
        if let Some(decision) = params.decision {
            adr.decision = decision.clone();
            body.decision = Some(decision);
            content_updated = true;
        }
        if let Some(consequences) = params.consequences {
            adr.consequences = consequences.clone();
            body.consequences = Some(consequences);
            content_updated = true;
        }

        // Set MADR metadata fields if provided.
        let metadata_fields_updated = params
            .decision_makers
            .as_ref()
            .is_some_and(|m| !m.is_empty())
            || params.consulted.as_ref().is_some_and(|c| !c.is_empty())
            || params.informed.as_ref().is_some_and(|i| !i.is_empty());

        if let Some(makers) = params.decision_makers
            && !makers.is_empty()
        {
            adr.set_decision_makers(makers);
            content_updated = true;
        }
        if let Some(consulted) = params.consulted
            && !consulted.is_empty()
        {
            adr.set_consulted(consulted);
            content_updated = true;
        }
        if let Some(informed) = params.informed
            && !informed.is_empty()
        {
            adr.set_informed(informed);
            content_updated = true;
        }

        // Handle tags.
        let tags_updated = params
            .tags
            .as_ref()
            .is_some_and(|tag_list| !tag_list.is_empty() && repo.config().is_next_gen());
        if let Some(tag_list) = params.tags
            && !tag_list.is_empty()
        {
            if repo.config().is_next_gen() {
                adr.set_tags(tag_list);
                content_updated = true;
            } else {
                warnings.push(
                    "Tags ignored: repository is not in NextGen mode. \
                     Use 'adrs --ng init' to enable NextGen mode."
                        .to_string(),
                );
            }
        }

        // Apply metadata and body updates separately so body patches never rewrite metadata.
        let metadata_updated = metadata_fields_updated || tags_updated;
        let body_updated = !body.is_empty();

        let mut final_path = path;
        if metadata_updated {
            final_path = repo.update_metadata(&adr).map_err(|e| e.to_string())?;
        }
        if body_updated {
            final_path = repo.update(&adr, body).map_err(|e| e.to_string())?;
        }

        #[derive(Serialize)]
        struct CreateResponse {
            message: String,
            number: u32,
            title: String,
            status: String,
            path: String,
            content_populated: bool,
            warnings: Vec<String>,
        }

        let response = CreateResponse {
            message: "ADR created successfully. Please review and edit as needed.".to_string(),
            number: adr.number,
            title: adr.title,
            status: adr.status.to_string(),
            path: final_path.display().to_string(),
            content_populated: content_updated,
            warnings,
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn update_status_impl(&self, params: UpdateStatusParams) -> Result<String, String> {
        let repo = self.open_repo()?;

        let status: AdrStatus = params.status.parse().unwrap(); // Infallible
        if matches!(status, AdrStatus::Custom(_)) {
            return Err(format!(
                "Invalid status '{}'. Valid values: proposed, accepted, deprecated, superseded",
                params.status
            ));
        }

        let path = repo
            .set_status(params.number, status.clone(), params.superseded_by)
            .map_err(|e| e.to_string())?;

        #[derive(Serialize)]
        struct StatusResponse {
            message: String,
            number: u32,
            new_status: String,
            path: String,
        }

        let response = StatusResponse {
            message: format!("Status updated to '{}'. Please verify the change.", status),
            number: params.number,
            new_status: status.to_string(),
            path: path.display().to_string(),
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn link_adrs_impl(&self, params: LinkAdrsParams) -> Result<String, String> {
        let repo = self.open_repo()?;

        let source_kind: LinkKind = params.link_type.parse().unwrap(); // Infallible
        if matches!(source_kind, LinkKind::Custom(_)) {
            return Err(format!(
                "Invalid link_type '{}'. Valid values: supersedes, superseded-by, amends, amended-by, relates-to",
                params.link_type
            ));
        }
        let target_kind = source_kind.reverse();

        repo.link(
            params.source,
            params.target,
            source_kind.clone(),
            target_kind.clone(),
        )
        .map_err(|e| e.to_string())?;

        #[derive(Serialize)]
        struct LinkResponse {
            message: String,
            source: u32,
            target: u32,
            source_link_type: String,
            target_link_type: String,
        }

        let response = LinkResponse {
            message: format!(
                "ADRs {} and {} linked successfully.",
                params.source, params.target
            ),
            source: params.source,
            target: params.target,
            source_link_type: source_kind.to_string(),
            target_link_type: target_kind.to_string(),
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn update_content_impl(&self, params: UpdateContentParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let mut adr = repo.get(params.number).map_err(|e| e.to_string())?;

        let mut body = adrs_core::BodySectionPatch::default();
        if let Some(context) = params.context {
            adr.context = context.clone();
            body.context = Some(context);
        }
        if let Some(decision) = params.decision {
            adr.decision = decision.clone();
            body.decision = Some(decision);
        }
        if let Some(consequences) = params.consequences {
            adr.consequences = consequences.clone();
            body.consequences = Some(consequences);
        }

        if body.is_empty() {
            return Err(
                "At least one of context, decision, or consequences must be provided".to_string(),
            );
        }

        let path = repo.update(&adr, body).map_err(|e| e.to_string())?;

        #[derive(Serialize)]
        struct ContentResponse {
            message: String,
            number: u32,
            path: String,
        }

        let response = ContentResponse {
            message: "ADR content updated. Please review the changes.".to_string(),
            number: params.number,
            path: path.display().to_string(),
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn update_tags_impl(&self, params: UpdateTagsParams) -> Result<String, String> {
        let repo = self.open_repo()?;

        if !repo.config().is_next_gen() {
            return Err("Tags require NextGen mode. Initialize with 'adrs --ng init'.".to_string());
        }

        let mut adr = repo.get(params.number).map_err(|e| e.to_string())?;

        if params.replace.unwrap_or(false) {
            adr.set_tags(params.tags.clone());
        } else {
            for tag in &params.tags {
                if !adr.tags.contains(tag) {
                    adr.add_tag(tag.clone());
                }
            }
        }

        let path = repo.update_metadata(&adr).map_err(|e| e.to_string())?;

        #[derive(Serialize)]
        struct TagsResponse {
            message: String,
            number: u32,
            tags: Vec<String>,
            path: String,
        }

        let response = TagsResponse {
            message: "ADR tags updated.".to_string(),
            number: params.number,
            tags: adr.tags,
            path: path.display().to_string(),
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn get_repository_info_impl(&self) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adrs = repo.list().map_err(|e| e.to_string())?;
        let config = repo.config();

        #[derive(Serialize)]
        struct RepoInfo {
            mode: String,
            adr_count: usize,
            adr_directory: String,
            statuses: StatusCounts,
        }

        #[derive(Serialize)]
        struct StatusCounts {
            proposed: usize,
            accepted: usize,
            deprecated: usize,
            superseded: usize,
            other: usize,
        }

        let mut counts = StatusCounts {
            proposed: 0,
            accepted: 0,
            deprecated: 0,
            superseded: 0,
            other: 0,
        };

        for adr in &adrs {
            match adr.status.to_string().to_lowercase().as_str() {
                "proposed" => counts.proposed += 1,
                "accepted" => counts.accepted += 1,
                "deprecated" => counts.deprecated += 1,
                "superseded" => counts.superseded += 1,
                _ => counts.other += 1,
            }
        }

        let info = RepoInfo {
            mode: if config.is_next_gen() {
                "nextgen".to_string()
            } else {
                "compatible".to_string()
            },
            adr_count: adrs.len(),
            adr_directory: config.adr_dir.display().to_string(),
            statuses: counts,
        };

        serde_json::to_string_pretty(&info).map_err(|e| e.to_string())
    }

    fn get_related_adrs_impl(&self, params: GetRelatedParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adr = repo.get(params.number).map_err(|e| e.to_string())?;
        let all_adrs = repo.list().map_err(|e| e.to_string())?;

        #[derive(Serialize)]
        struct RelatedAdr {
            number: u32,
            title: String,
            link_type: String,
            direction: String,
        }

        #[derive(Serialize)]
        struct RelatedResponse {
            number: u32,
            title: String,
            related: Vec<RelatedAdr>,
        }

        let mut related = Vec::new();

        // Outgoing links from this ADR
        for link in &adr.links {
            if let Some(target) = all_adrs.iter().find(|a| a.number == link.target) {
                related.push(RelatedAdr {
                    number: target.number,
                    title: target.title.clone(),
                    link_type: link.kind.to_string(),
                    direction: "outgoing".to_string(),
                });
            }
        }

        // Incoming links to this ADR
        for other in &all_adrs {
            if other.number == params.number {
                continue;
            }
            for link in &other.links {
                if link.target == params.number {
                    related.push(RelatedAdr {
                        number: other.number,
                        title: other.title.clone(),
                        link_type: link.kind.to_string(),
                        direction: "incoming".to_string(),
                    });
                }
            }
        }

        let response = RelatedResponse {
            number: adr.number,
            title: adr.title,
            related,
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn validate_adr_impl(&self, params: ValidateAdrParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adr = repo.get(params.number).map_err(|e| e.to_string())?;
        let all_adrs = repo.list().map_err(|e| e.to_string())?;

        let mut issues = Vec::new();

        // Check sections
        let context_present = !adr.context.is_empty();
        let context_empty = adr.context.trim().is_empty();
        let decision_present = !adr.decision.is_empty();
        let decision_empty = adr.decision.trim().is_empty();
        let consequences_present = !adr.consequences.is_empty();
        let consequences_empty = adr.consequences.trim().is_empty();

        // Report missing sections
        if !context_present || context_empty {
            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                message: "Context section is missing or empty".to_string(),
                section: Some("context".to_string()),
            });
        }

        if !decision_present || decision_empty {
            issues.push(ValidationIssue {
                severity: "error".to_string(),
                message: "Decision section is missing or empty".to_string(),
                section: Some("decision".to_string()),
            });
        }

        if !consequences_present || consequences_empty {
            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                message: "Consequences section is missing or empty".to_string(),
                section: Some("consequences".to_string()),
            });
        }

        // Check title
        if adr.title.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: "error".to_string(),
                message: "ADR title is empty".to_string(),
                section: None,
            });
        }

        // Validate links reference existing ADRs
        for link in &adr.links {
            if !all_adrs.iter().any(|a| a.number == link.target) {
                issues.push(ValidationIssue {
                    severity: "error".to_string(),
                    message: format!("Link references non-existent ADR #{}", link.target),
                    section: None,
                });
            }
        }

        let valid = !issues.iter().any(|i| i.severity == "error");

        let result = ValidationResult {
            number: adr.number,
            title: adr.title,
            valid,
            issues,
            sections: SectionStatus {
                context: SectionInfo {
                    present: context_present,
                    empty: context_empty,
                },
                decision: SectionInfo {
                    present: decision_present,
                    empty: decision_empty,
                },
                consequences: SectionInfo {
                    present: consequences_present,
                    empty: consequences_empty,
                },
            },
        };

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn get_adr_sections_impl(&self, params: GetAdrSectionsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adr = repo.get(params.number).map_err(|e| e.to_string())?;

        let sections = AdrSections {
            number: adr.number,
            title: adr.title.clone(),
            status: adr.status.to_string(),
            date: Some(adr.date.to_string()),
            tags: adr.tags.clone(),
            context: adr.context.clone(),
            decision: adr.decision.clone(),
            consequences: adr.consequences.clone(),
            links: adr
                .links
                .iter()
                .map(|l| LinkInfo {
                    kind: l.kind.to_string(),
                    target: l.target,
                    description: l.description.clone(),
                })
                .collect(),
        };

        serde_json::to_string_pretty(&sections).map_err(|e| e.to_string())
    }

    fn compare_adrs_impl(&self, params: CompareAdrsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let source_adr = repo.get(params.source).map_err(|e| e.to_string())?;
        let target_adr = repo.get(params.target).map_err(|e| e.to_string())?;

        let result = CompareResult {
            source: AdrBrief {
                number: source_adr.number,
                title: source_adr.title.clone(),
                status: source_adr.status.to_string(),
            },
            target: AdrBrief {
                number: target_adr.number,
                title: target_adr.title.clone(),
                status: target_adr.status.to_string(),
            },
            differences: Differences {
                title_changed: source_adr.title != target_adr.title,
                status_changed: source_adr.status.to_string() != target_adr.status.to_string(),
                context: SectionDiff {
                    changed: source_adr.context != target_adr.context,
                    source_empty: source_adr.context.trim().is_empty(),
                    target_empty: target_adr.context.trim().is_empty(),
                    source_length: source_adr.context.len(),
                    target_length: target_adr.context.len(),
                },
                decision: SectionDiff {
                    changed: source_adr.decision != target_adr.decision,
                    source_empty: source_adr.decision.trim().is_empty(),
                    target_empty: target_adr.decision.trim().is_empty(),
                    source_length: source_adr.decision.len(),
                    target_length: target_adr.decision.len(),
                },
                consequences: SectionDiff {
                    changed: source_adr.consequences != target_adr.consequences,
                    source_empty: source_adr.consequences.trim().is_empty(),
                    target_empty: target_adr.consequences.trim().is_empty(),
                    source_length: source_adr.consequences.len(),
                    target_length: target_adr.consequences.len(),
                },
            },
        };

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn bulk_update_status_impl(&self, params: BulkUpdateStatusParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let status: AdrStatus = params.status.parse().unwrap(); // Infallible
        if matches!(status, AdrStatus::Custom(_)) {
            return Err(format!(
                "Invalid status '{}'. Valid values: proposed, accepted, deprecated, superseded",
                params.status
            ));
        }

        let mut updated = Vec::new();
        let mut failed = Vec::new();

        for number in params.numbers {
            match repo.get(number) {
                Ok(adr) => {
                    let old_status = adr.status.to_string();
                    match repo.set_status(number, status.clone(), params.superseded_by) {
                        Ok(_) => {
                            updated.push(UpdatedAdr {
                                number,
                                old_status,
                                new_status: status.to_string(),
                            });
                        }
                        Err(e) => {
                            failed.push(FailedAdr {
                                number,
                                error: e.to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    failed.push(FailedAdr {
                        number,
                        error: e.to_string(),
                    });
                }
            }
        }

        let summary = if failed.is_empty() {
            format!(
                "Successfully updated {} ADR(s) to '{}'",
                updated.len(),
                status
            )
        } else {
            format!("Updated {} ADR(s), {} failed", updated.len(), failed.len())
        };

        let result = BulkUpdateResult {
            updated,
            failed,
            summary,
        };

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn suggest_tags_impl(&self, params: SuggestTagsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adr = repo.get(params.number).map_err(|e| e.to_string())?;
        let max_tags = params.max_tags.unwrap_or(5);

        let text = format!(
            "{} {} {} {}",
            adr.title, adr.context, adr.decision, adr.consequences
        )
        .to_lowercase();

        let categories: Vec<(&str, Vec<&str>)> = vec![
            (
                "security",
                vec![
                    "security",
                    "auth",
                    "authentication",
                    "authorization",
                    "encrypt",
                    "password",
                    "token",
                    "oauth",
                    "jwt",
                    "permission",
                    "access control",
                ],
            ),
            (
                "database",
                vec![
                    "database",
                    "sql",
                    "postgres",
                    "mysql",
                    "mongodb",
                    "redis",
                    "schema",
                    "migration",
                    "query",
                    "orm",
                    "persistence",
                    "storage",
                ],
            ),
            (
                "api",
                vec![
                    "api",
                    "rest",
                    "graphql",
                    "endpoint",
                    "http",
                    "grpc",
                    "websocket",
                    "request",
                    "response",
                    "route",
                ],
            ),
            (
                "testing",
                vec![
                    "test",
                    "testing",
                    "unit test",
                    "integration",
                    "e2e",
                    "mock",
                    "fixture",
                    "coverage",
                    "tdd",
                    "bdd",
                ],
            ),
            (
                "infrastructure",
                vec![
                    "infrastructure",
                    "deploy",
                    "docker",
                    "kubernetes",
                    "k8s",
                    "ci/cd",
                    "pipeline",
                    "cloud",
                    "aws",
                    "gcp",
                    "azure",
                    "terraform",
                ],
            ),
            (
                "performance",
                vec![
                    "performance",
                    "cache",
                    "caching",
                    "optimization",
                    "latency",
                    "throughput",
                    "scalability",
                    "load",
                    "benchmark",
                ],
            ),
            (
                "architecture",
                vec![
                    "architecture",
                    "pattern",
                    "microservice",
                    "monolith",
                    "modular",
                    "layer",
                    "component",
                    "design",
                    "structure",
                ],
            ),
            (
                "frontend",
                vec![
                    "frontend",
                    "ui",
                    "ux",
                    "react",
                    "vue",
                    "angular",
                    "css",
                    "html",
                    "component",
                    "browser",
                ],
            ),
            (
                "documentation",
                vec![
                    "documentation",
                    "docs",
                    "readme",
                    "wiki",
                    "guide",
                    "tutorial",
                    "reference",
                    "markdown",
                ],
            ),
            (
                "tooling",
                vec![
                    "tooling",
                    "cli",
                    "tool",
                    "linter",
                    "formatter",
                    "build",
                    "compile",
                    "bundle",
                ],
            ),
        ];

        let mut suggestions: Vec<SuggestedTag> = categories
            .iter()
            .filter_map(|(tag, keywords)| {
                let matches: Vec<&str> = keywords
                    .iter()
                    .filter(|kw| text.contains(*kw))
                    .copied()
                    .collect();

                if matches.is_empty() {
                    None
                } else {
                    let confidence = (matches.len() as f32 / keywords.len() as f32).min(1.0);
                    let reason = format!("Found keywords: {}", matches.join(", "));
                    Some(SuggestedTag {
                        tag: tag.to_string(),
                        confidence,
                        reason,
                    })
                }
            })
            .collect();

        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(max_tags);
        suggestions.retain(|s| !adr.tags.iter().any(|t| t.to_lowercase() == s.tag));

        let result = SuggestTagsResult {
            number: adr.number,
            title: adr.title,
            existing_tags: adr.tags,
            suggested_tags: suggestions,
        };

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn run_doctor_impl(&self) -> Result<String, String> {
        let repo = self.open_repo()?;
        let report = check_all(&repo).map_err(|e| e.to_string())?;

        let error_count = report.count_by_severity(IssueSeverity::Error);
        let warning_count = report.count_by_severity(IssueSeverity::Warning);
        let info_count = report.count_by_severity(IssueSeverity::Info);

        let issues: Vec<DoctorIssue> = report
            .issues
            .iter()
            .map(|issue| DoctorIssue {
                severity: issue.severity.to_string(),
                rule_id: issue.rule_id.clone(),
                rule_name: issue.rule_name.clone(),
                message: issue.message.clone(),
                path: issue.path.as_ref().map(|p| p.display().to_string()),
                line: issue.line,
                adr_number: issue.adr_number,
            })
            .collect();

        let result = DoctorResult {
            healthy: error_count == 0,
            error_count,
            warning_count,
            info_count,
            issues,
        };

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn export_adrs_impl(&self, params: ExportAdrsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let mut export = export_repository(&repo).map_err(|e| e.to_string())?;

        // Filter by numbers if provided
        if let Some(ref numbers) = params.numbers {
            export.adrs.retain(|adr| numbers.contains(&adr.number));
        }

        // Strip content fields if metadata_only
        if params.metadata_only.unwrap_or(false) {
            for adr in &mut export.adrs {
                adr.context = None;
                adr.decision = None;
                adr.consequences = None;
                adr.confirmation = None;
                adr.decision_drivers.clear();
                adr.considered_options.clear();
                adr.custom_sections.clear();
            }
        }

        serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
    }
}

/// Run the MCP server on stdio.
pub async fn serve_stdio(root: PathBuf) -> Result<()> {
    use tower_mcp::StdioTransport;

    let router = build_router(root);
    let mut transport = StdioTransport::new(router);
    transport.run().await?;
    Ok(())
}

/// Run the MCP server over HTTP.
#[cfg(feature = "mcp-http")]
pub async fn serve_http(root: PathBuf, addr: std::net::SocketAddr) -> Result<()> {
    use tower_mcp::HttpTransport;

    let router = build_router(root);

    eprintln!("MCP server listening on http://{}/mcp", addr);

    let transport = HttpTransport::new(router);
    transport.serve(&addr.to_string()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tower_mcp::client::{ChannelTransport, McpClient};

    /// Helper: create an initialized MCP client connected to a temp ADR repo.
    async fn setup_client(ng: bool) -> (McpClient, tempfile::TempDir) {
        let temp = tempfile::tempdir().unwrap();
        adrs_core::Repository::init(temp.path(), None, ng).unwrap();
        let router = build_router(temp.path().to_path_buf());
        let transport = ChannelTransport::new(router);
        let client = McpClient::connect(transport).await.unwrap();
        client.initialize("test-client", "1.0.0").await.unwrap();
        (client, temp)
    }

    #[tokio::test]
    async fn test_initialize_returns_server_info() {
        let (client, _tmp) = setup_client(false).await;
        let info = client.server_info().await.unwrap();
        assert_eq!(info.server_info.name, "adrs");
        assert!(info.capabilities.tools.is_some());
    }

    #[tokio::test]
    async fn test_list_tools_returns_all_17() {
        let (client, _tmp) = setup_client(false).await;
        let tools = client.list_all_tools().await.unwrap();
        assert_eq!(tools.len(), 17, "expected 17 tools, got {}", tools.len());

        let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"list_adrs"));
        assert!(names.contains(&"get_adr"));
        assert!(names.contains(&"search_adrs"));
        assert!(names.contains(&"create_adr"));
        assert!(names.contains(&"update_status"));
        assert!(names.contains(&"link_adrs"));
        assert!(names.contains(&"update_content"));
        assert!(names.contains(&"update_tags"));
        assert!(names.contains(&"get_repository_info"));
        assert!(names.contains(&"get_related_adrs"));
        assert!(names.contains(&"validate_adr"));
        assert!(names.contains(&"get_adr_sections"));
        assert!(names.contains(&"compare_adrs"));
        assert!(names.contains(&"bulk_update_status"));
        assert!(names.contains(&"suggest_tags"));
        assert!(names.contains(&"run_doctor"));
        assert!(names.contains(&"export_adrs"));
    }

    #[tokio::test]
    async fn test_read_only_tools_have_annotation() {
        let (client, _tmp) = setup_client(false).await;
        let tools = client.list_all_tools().await.unwrap();

        let read_only_names = [
            "list_adrs",
            "get_adr",
            "search_adrs",
            "get_repository_info",
            "get_related_adrs",
            "validate_adr",
            "get_adr_sections",
            "compare_adrs",
            "suggest_tags",
            "run_doctor",
            "export_adrs",
        ];

        for name in read_only_names {
            let tool = tools
                .iter()
                .find(|t| t.name == name)
                .unwrap_or_else(|| panic!("tool {name} not found"));
            let annotations = tool
                .annotations
                .as_ref()
                .unwrap_or_else(|| panic!("tool {name} has no annotations"));
            assert!(
                annotations.read_only_hint,
                "tool {name} should be read-only"
            );
        }
    }

    #[tokio::test]
    async fn test_list_adrs_returns_initial_adr() {
        let (client, _tmp) = setup_client(false).await;
        let result = client.call_tool_text("list_adrs", json!({})).await.unwrap();

        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert_eq!(adrs.len(), 1);
        assert_eq!(adrs[0].number, 1);
        assert_eq!(adrs[0].status, "Accepted");
    }

    #[tokio::test]
    async fn test_get_adr_returns_content() {
        let (client, _tmp) = setup_client(false).await;
        let result = client
            .call_tool_text("get_adr", json!({"number": 1}))
            .await
            .unwrap();

        let adr: AdrDetail = serde_json::from_str(&result).unwrap();
        assert_eq!(adr.number, 1);
        assert!(!adr.content.is_empty());
    }

    #[tokio::test]
    async fn test_create_adr_and_get() {
        let (client, _tmp) = setup_client(false).await;

        // Create a new ADR
        let result = client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use PostgreSQL",
                    "context": "We need a database.",
                    "decision": "Use PostgreSQL.",
                    "consequences": "Need DBA skills."
                }),
            )
            .await
            .unwrap();

        let created: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(created["number"], 2);
        assert_eq!(created["status"], "Proposed");

        // Retrieve it back
        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();

        let adr: AdrDetail = serde_json::from_str(&result).unwrap();
        assert_eq!(adr.title, "Use PostgreSQL");
    }

    #[tokio::test]
    async fn test_search_adrs() {
        let (client, _tmp) = setup_client(false).await;

        // Create an ADR with searchable content
        client
            .call_tool_text(
                "create_adr",
                json!({"title": "Adopt Kubernetes for deployment"}),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("search_adrs", json!({"query": "Kubernetes"}))
            .await
            .unwrap();

        let matches: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!matches.is_empty(), "search should find Kubernetes ADR");
        assert!(
            matches
                .iter()
                .any(|m| m["title"].as_str().unwrap().contains("Kubernetes"))
        );
    }

    #[tokio::test]
    async fn test_update_status() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Use Redis"}))
            .await
            .unwrap();

        let result = client
            .call_tool_text("update_status", json!({"number": 2, "status": "accepted"}))
            .await
            .unwrap();

        let updated: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(updated["new_status"], "Accepted");

        // Verify via get
        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();
        assert_eq!(adr.status, "Accepted");
    }

    #[tokio::test]
    async fn test_link_adrs() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Use MySQL"}))
            .await
            .unwrap();
        client
            .call_tool_text("create_adr", json!({"title": "Use PostgreSQL"}))
            .await
            .unwrap();

        let result = client
            .call_tool_text(
                "link_adrs",
                json!({"source": 3, "target": 2, "link_type": "Supersedes"}),
            )
            .await
            .unwrap();
        assert!(result.contains("Supersedes"));

        // Verify the link via get_related_adrs
        let result = client
            .call_tool_text("get_related_adrs", json!({"number": 3}))
            .await
            .unwrap();
        let related: serde_json::Value = serde_json::from_str(&result).unwrap();
        let links = related["related"].as_array().unwrap();
        assert!(
            links.iter().any(|l| l["direction"] == "outgoing"),
            "expected outgoing link"
        );
    }

    #[tokio::test]
    async fn test_update_content() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Use Docker"}))
            .await
            .unwrap();

        client
            .call_tool_text(
                "update_content",
                json!({
                    "number": 2,
                    "context": "We need containerization.",
                    "decision": "Use Docker."
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr_sections", json!({"number": 2}))
            .await
            .unwrap();
        let sections: AdrSections = serde_json::from_str(&result).unwrap();
        assert!(sections.context.contains("containerization"));
        assert!(sections.decision.contains("Docker"));
    }

    #[tokio::test]
    async fn test_update_content_rejects_empty_body_patch() {
        // Progressive baseline / contract pin (PR #311 review 4707905821):
        // update_content with only number must error, not silently no-op.
        let (client, _tmp) = setup_client(false).await;
        client
            .call_tool_text("create_adr", json!({"title": "Empty patch target"}))
            .await
            .unwrap();

        let result = client
            .call_tool("update_content", json!({"number": 2}))
            .await
            .unwrap();
        assert!(
            result.is_error,
            "empty update_content must reject: {:?}",
            result
        );
        let text = format!("{:?}", result);
        assert!(
            text.contains("At least one of context, decision, or consequences")
                || text.contains("must be provided"),
            "error should mention required body fields: {text}"
        );
    }

    #[tokio::test]
    async fn test_update_content_madr_preserves_decision() {
        // MADR 4.0.0 ADRs must not be re-rendered as Nygard/adr-tools on update_content.
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "format": "madr",
                    "context": "Original context about caching needs.",
                    "decision": "Chosen option: \"Redis\", because it is fast."
                }),
            )
            .await
            .unwrap();

        client
            .call_tool_text(
                "update_content",
                json!({
                    "number": 2,
                    "context": "Updated context text."
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();

        assert!(
            adr.content.contains("Updated context text."),
            "context should be updated, got:\n{}",
            adr.content
        );
        assert!(
            adr.content.contains("Chosen option: \"Redis\""),
            "decision should be preserved, got:\n{}",
            adr.content
        );
        assert!(
            adr.content.contains("Context and Problem Statement"),
            "MADR 4.0.0 heading should be preserved, got:\n{}",
            adr.content
        );
        assert!(
            adr.content.contains("Decision Outcome"),
            "MADR 4.0.0 heading should be preserved, got:\n{}",
            adr.content
        );
        assert!(
            !adr.content
                .contains("What is the change that we're proposing"),
            "Nygard/adr-tools placeholder should not appear, got:\n{}",
            adr.content
        );
    }

    #[tokio::test]
    async fn test_update_content_madr_preserves_h3_subsections() {
        let (client, tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "format": "madr",
                    "context": "Original context.",
                    "decision": "Chosen option: \"Redis\", because it is fast."
                }),
            )
            .await
            .unwrap();

        let adr_path = tmp.path().join("doc/adr/0002-use-redis-for-caching.md");
        let rich_content = r#"---
number: 2
title: Use Redis for caching
date: 2026-01-15
status: proposed
---

## Context and Problem Statement

Original context.

## Decision Outcome

Chosen option: "Redis", because it is fast.

### Consequences

* Good, because it provides pub/sub
* Bad, because it needs memory

### Confirmation

We will confirm via load tests.
"#;
        std::fs::write(&adr_path, rich_content).unwrap();

        client
            .call_tool_text(
                "update_content",
                json!({
                    "number": 2,
                    "context": "Updated context text."
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();

        assert!(adr.content.contains("Updated context text."));
        assert!(adr.content.contains("### Consequences"));
        assert!(adr.content.contains("* Good, because it provides pub/sub"));
        assert!(adr.content.contains("* Bad, because it needs memory"));
        assert!(adr.content.contains("### Confirmation"));
        assert!(adr.content.contains("We will confirm via load tests."));
    }

    #[tokio::test]
    async fn test_create_adr_madr_with_consequences() {
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "format": "madr",
                    "context": "We need caching.",
                    "decision": "Chosen option: \"Redis\", because it is fast.",
                    "consequences": "* Good, because it provides pub/sub"
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();

        assert!(adr.content.contains("### Consequences"));
        assert!(adr.content.contains("* Good, because it provides pub/sub"));
    }

    #[tokio::test]
    async fn test_update_content_madr_consequences_only() {
        let (client, tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "format": "madr",
                    "context": "Context.",
                    "decision": "Chosen option: \"Redis\", because it is fast."
                }),
            )
            .await
            .unwrap();

        let adr_path = tmp.path().join("doc/adr/0002-use-redis-for-caching.md");
        let rich_content = r#"---
number: 2
title: Use Redis for caching
date: 2026-01-15
status: proposed
---

## Context and Problem Statement

Context.

## Decision Outcome

Chosen option: "Redis", because it is fast.

### Consequences

* Old consequence
"#;
        std::fs::write(&adr_path, rich_content).unwrap();

        client
            .call_tool_text(
                "update_content",
                json!({
                    "number": 2,
                    "consequences": "* New consequence"
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();

        assert!(
            adr.content
                .contains("Chosen option: \"Redis\", because it is fast.")
        );
        assert!(adr.content.contains("* New consequence"));
        assert!(!adr.content.contains("* Old consequence"));
    }

    #[tokio::test]
    async fn test_update_content_nygard_consequences_only() {
        // Nygard ADRs use top-level ## Consequences; consequences-only updates must
        // patch that H2 and leave ## Decision untouched (no MADR ### injection).
        let (client, tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "context": "Context.",
                    "decision": "Original decision."
                }),
            )
            .await
            .unwrap();

        let adr_path = tmp.path().join("doc/adr/0002-use-redis-for-caching.md");
        let content = r#"---
number: 2
title: Use Redis for caching
date: 2026-01-15
status: proposed
---

## Context

Context.

## Decision

Original decision.

## Consequences

* Old item
"#;
        std::fs::write(&adr_path, content).unwrap();

        client
            .call_tool_text(
                "update_content",
                json!({
                    "number": 2,
                    "consequences": "* New item"
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();

        assert!(adr.content.contains("Original decision."));
        assert!(adr.content.contains("## Consequences"));
        assert!(adr.content.contains("* New item"));
        assert!(!adr.content.contains("* Old item"));
        assert!(
            !adr.content.contains("### Consequences"),
            "Nygard ADR should not get MADR ### Consequences under Decision, got:\n{}",
            adr.content
        );
    }

    #[tokio::test]
    async fn test_update_status_madr_preserves_rich_body() {
        let (client, tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "format": "madr",
                    "context": "Context.",
                    "decision": "Chosen option: \"Redis\", because it is fast."
                }),
            )
            .await
            .unwrap();

        let adr_path = tmp.path().join("doc/adr/0002-use-redis-for-caching.md");
        let rich_content = r#"---
number: 2
title: Use Redis for caching
date: 2026-01-15
status: proposed
---

## Context and Problem Statement

Context.

## Decision Outcome

Chosen option: "Redis", because it is fast.

### Consequences

* Good item

### Confirmation

Confirm via tests.
"#;
        std::fs::write(&adr_path, rich_content).unwrap();

        client
            .call_tool_text("update_status", json!({"number": 2, "status": "accepted"}))
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();
        let adr: AdrDetail = serde_json::from_str(&result).unwrap();

        assert!(
            adr.content.contains("status: accepted") || adr.status.eq_ignore_ascii_case("accepted")
        );
        assert!(adr.content.contains("### Consequences"));
        assert!(adr.content.contains("* Good item"));
        assert!(adr.content.contains("### Confirmation"));
        assert!(adr.content.contains("Confirm via tests."));
    }

    #[tokio::test]
    async fn test_update_tags_ng_mode() {
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text("create_adr", json!({"title": "Use gRPC"}))
            .await
            .unwrap();

        client
            .call_tool_text(
                "update_tags",
                json!({"number": 2, "tags": ["api", "networking"]}),
            )
            .await
            .unwrap();

        // Verify tags via list with filter
        let result = client
            .call_tool_text("list_adrs", json!({"tag": "api"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert!(adrs.iter().any(|a| a.number == 2));
    }

    #[tokio::test]
    async fn test_get_repository_info() {
        let (client, _tmp) = setup_client(true).await;
        let result = client
            .call_tool_text("get_repository_info", json!({}))
            .await
            .unwrap();

        let info: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(info["mode"], "nextgen");
        assert!(info["adr_count"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_validate_adr() {
        let (client, _tmp) = setup_client(false).await;
        let result = client
            .call_tool_text("validate_adr", json!({"number": 1}))
            .await
            .unwrap();

        let validation: ValidationResult = serde_json::from_str(&result).unwrap();
        assert_eq!(validation.number, 1);
    }

    #[tokio::test]
    async fn test_compare_adrs() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Use REST"}))
            .await
            .unwrap();

        let result = client
            .call_tool_text("compare_adrs", json!({"source": 1, "target": 2}))
            .await
            .unwrap();

        let cmp: CompareResult = serde_json::from_str(&result).unwrap();
        assert_eq!(cmp.source.number, 1);
        assert_eq!(cmp.target.number, 2);
        assert!(cmp.differences.title_changed);
    }

    #[tokio::test]
    async fn test_bulk_update_status() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "ADR A"}))
            .await
            .unwrap();
        client
            .call_tool_text("create_adr", json!({"title": "ADR B"}))
            .await
            .unwrap();

        let result = client
            .call_tool_text(
                "bulk_update_status",
                json!({"numbers": [2, 3], "status": "accepted"}),
            )
            .await
            .unwrap();

        let bulk: BulkUpdateResult = serde_json::from_str(&result).unwrap();
        assert_eq!(bulk.updated.len(), 2);
        assert!(bulk.failed.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_tags() {
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use PostgreSQL for database",
                    "context": "We need a relational database for storing user data and API records.",
                    "decision": "Use PostgreSQL as the primary database."
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("suggest_tags", json!({"number": 2}))
            .await
            .unwrap();

        let suggestions: SuggestTagsResult = serde_json::from_str(&result).unwrap();
        assert_eq!(suggestions.number, 2);
        assert!(!suggestions.suggested_tags.is_empty());
    }

    #[tokio::test]
    async fn test_list_adrs_filter_by_status() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Proposed ADR"}))
            .await
            .unwrap();

        // Filter for accepted (only the init ADR)
        let result = client
            .call_tool_text("list_adrs", json!({"status": "accepted"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert_eq!(adrs.len(), 1);
        assert_eq!(adrs[0].number, 1);

        // Filter for proposed (only the new one)
        let result = client
            .call_tool_text("list_adrs", json!({"status": "proposed"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert_eq!(adrs.len(), 1);
        assert_eq!(adrs[0].number, 2);
    }

    #[tokio::test]
    async fn test_get_nonexistent_adr_returns_error() {
        let (client, _tmp) = setup_client(false).await;
        let result = client
            .call_tool("get_adr", json!({"number": 999}))
            .await
            .unwrap();

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_update_tags_compatible_mode_error() {
        // setup_client(false) creates a compatible (non-NG) repo
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "ADR for tags test"}))
            .await
            .unwrap();

        let result = client
            .call_tool(
                "update_tags",
                json!({"number": 2, "tags": ["architecture"]}),
            )
            .await
            .unwrap();

        assert!(
            result.is_error,
            "update_tags should fail in compatible (non-NG) mode"
        );
    }

    #[tokio::test]
    async fn test_validate_adr_invalid_adr() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Incomplete ADR"}))
            .await
            .unwrap();

        // Blank the sections: " " trims to empty for validation but is truthy
        // to the template renderer (so it writes the space, not the placeholder).
        client
            .call_tool_text(
                "update_content",
                json!({"number": 2, "context": " ", "decision": " ", "consequences": " "}),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("validate_adr", json!({"number": 2}))
            .await
            .unwrap();

        let validation: ValidationResult = serde_json::from_str(&result).unwrap();
        assert_eq!(validation.number, 2);
        assert!(
            !validation.issues.is_empty(),
            "ADR with blank sections should have validation issues"
        );
    }

    #[tokio::test]
    async fn test_update_status_rejects_invalid() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "Test ADR"}))
            .await
            .unwrap();

        let result = client
            .call_tool("update_status", json!({"number": 2, "status": "banana"}))
            .await
            .unwrap();

        assert!(
            result.is_error,
            "update_status should reject unknown status 'banana'"
        );
    }

    #[tokio::test]
    async fn test_link_adrs_rejects_invalid_link_type() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "ADR A"}))
            .await
            .unwrap();
        client
            .call_tool_text("create_adr", json!({"title": "ADR B"}))
            .await
            .unwrap();

        let result = client
            .call_tool(
                "link_adrs",
                json!({"source": 2, "target": 3, "link_type": "banana"}),
            )
            .await
            .unwrap();

        assert!(
            result.is_error,
            "link_adrs should reject unknown link_type 'banana'"
        );
    }

    #[tokio::test]
    async fn test_bulk_update_status_rejects_invalid() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "ADR A"}))
            .await
            .unwrap();

        let result = client
            .call_tool(
                "bulk_update_status",
                json!({"numbers": [2], "status": "banana"}),
            )
            .await
            .unwrap();

        assert!(
            result.is_error,
            "bulk_update_status should reject unknown status 'banana'"
        );
    }

    #[tokio::test]
    async fn test_ping() {
        let (client, _tmp) = setup_client(false).await;
        client.ping().await.unwrap();
    }

    // ========== New tests for #230 and #234 ==========

    #[tokio::test]
    async fn test_create_adr_madr_format() {
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use MADR format",
                    "format": "madr"
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();

        let adr: AdrDetail = serde_json::from_str(&result).unwrap();
        assert!(
            adr.content.contains("Context and Problem Statement"),
            "MADR format ADR should contain 'Context and Problem Statement', got:\n{}",
            adr.content
        );
    }

    #[tokio::test]
    async fn test_create_adr_madr_with_decision_makers() {
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use MADR with decision makers",
                    "format": "madr",
                    "decision_makers": ["Alice", "Bob"]
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("get_adr", json!({"number": 2}))
            .await
            .unwrap();

        let adr: AdrDetail = serde_json::from_str(&result).unwrap();
        assert!(
            adr.content.contains("Alice"),
            "MADR ADR content should contain 'Alice', got:\n{}",
            adr.content
        );
        assert!(
            adr.content.contains("Bob"),
            "MADR ADR content should contain 'Bob', got:\n{}",
            adr.content
        );
    }

    #[tokio::test]
    async fn test_create_adr_with_tags_ng_mode() {
        let (client, _tmp) = setup_client(true).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use Redis for caching",
                    "tags": ["api", "security"]
                }),
            )
            .await
            .unwrap();

        // Verify tag filter works for the newly created ADR
        let result = client
            .call_tool_text("list_adrs", json!({"tag": "api"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert!(
            adrs.iter().any(|a| a.number == 2),
            "ADR #2 should appear when filtering by tag 'api'"
        );
    }

    #[tokio::test]
    async fn test_create_adr_with_tags_compatible_mode_warns() {
        // setup_client(false) creates a compatible (non-NG) repo
        let (client, _tmp) = setup_client(false).await;

        let result = client
            .call_tool(
                "create_adr",
                json!({
                    "title": "ADR with tags in compat mode",
                    "tags": ["api"]
                }),
            )
            .await
            .unwrap();

        // Must NOT be an error
        assert!(
            !result.is_error,
            "create_adr with tags in compatible mode should warn, not error"
        );

        // Response JSON should include a warnings array with at least one entry
        let text = result.all_text();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        let warnings = value["warnings"].as_array().unwrap();
        assert!(
            !warnings.is_empty(),
            "response should contain at least one warning when tags are given in compatible mode"
        );
    }

    #[tokio::test]
    async fn test_create_adr_madr_variant_bare() {
        // The MADR bare template uses empty YAML fields (decision-makers:, consulted:,
        // informed: with no values), which the current parser cannot round-trip. This
        // test therefore only verifies that create_adr accepts format=madr + variant=bare
        // and succeeds (produces a file at the returned path). A full read-back test is
        // not feasible here until the parser handles null YAML values.
        let (client, _tmp) = setup_client(true).await;

        let result = client
            .call_tool(
                "create_adr",
                json!({
                    "title": "MADR bare variant ADR",
                    "format": "madr",
                    "variant": "bare"
                }),
            )
            .await
            .unwrap();

        assert!(
            !result.is_error,
            "create_adr with format=madr and variant=bare should succeed"
        );

        let text = result.all_text();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["number"], 2, "created ADR should be number 2");
        assert_eq!(
            value["status"], "Proposed",
            "created ADR should be Proposed"
        );
    }

    #[tokio::test]
    async fn test_create_adr_invalid_format() {
        let (client, _tmp) = setup_client(false).await;

        let result = client
            .call_tool(
                "create_adr",
                json!({
                    "title": "Invalid format test",
                    "format": "bogus"
                }),
            )
            .await
            .unwrap();

        assert!(
            result.is_error,
            "create_adr with invalid format should return an error"
        );
    }

    #[tokio::test]
    async fn test_create_adr_invalid_variant() {
        let (client, _tmp) = setup_client(false).await;

        let result = client
            .call_tool(
                "create_adr",
                json!({
                    "title": "Invalid variant test",
                    "variant": "bogus"
                }),
            )
            .await
            .unwrap();

        assert!(
            result.is_error,
            "create_adr with invalid variant should return an error"
        );
    }

    // ========== New tests for #231, #232, #233 ==========

    #[tokio::test]
    async fn test_list_adrs_filter_by_since() {
        let (client, _tmp) = setup_client(false).await;
        // Filter by a far-future date -- should return no ADRs
        let result = client
            .call_tool_text("list_adrs", json!({"since": "2099-01-01"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert_eq!(adrs.len(), 0, "no ADRs should be on or after 2099-01-01");
    }

    #[tokio::test]
    async fn test_list_adrs_filter_by_until() {
        let (client, _tmp) = setup_client(false).await;
        // Filter by a past date -- the init ADR (today) should not appear
        let result = client
            .call_tool_text("list_adrs", json!({"until": "2000-01-01"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert_eq!(adrs.len(), 0, "no ADRs should be on or before 2000-01-01");
    }

    #[tokio::test]
    async fn test_list_adrs_filter_by_decider() {
        let (client, _tmp) = setup_client(true).await;
        // Create an ADR with decision_makers via MADR format
        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use gRPC",
                    "format": "madr",
                    "decision_makers": ["Alice Smith", "Bob Jones"]
                }),
            )
            .await
            .unwrap();

        // Filter by decider "alice" -- should find the new ADR
        let result = client
            .call_tool_text("list_adrs", json!({"decider": "alice"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert!(
            adrs.iter().any(|a| a.number == 2),
            "should find ADR #2 with decider 'alice'"
        );

        // Filter by decider "charlie" -- should return empty
        let result = client
            .call_tool_text("list_adrs", json!({"decider": "charlie"}))
            .await
            .unwrap();
        let adrs: Vec<AdrSummary> = serde_json::from_str(&result).unwrap();
        assert_eq!(adrs.len(), 0, "no ADRs with decider 'charlie'");
    }

    #[tokio::test]
    async fn test_list_adrs_invalid_since_date() {
        let (client, _tmp) = setup_client(false).await;
        let result = client
            .call_tool("list_adrs", json!({"since": "not-a-date"}))
            .await
            .unwrap();
        assert!(result.is_error, "invalid since date should return an error");
    }

    #[tokio::test]
    async fn test_search_adrs_case_sensitive() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Use PostgreSQL for Database",
                    "context": "PostgreSQL is the chosen database."
                }),
            )
            .await
            .unwrap();

        // Case-insensitive (default) -- should match
        let result = client
            .call_tool_text("search_adrs", json!({"query": "postgresql"}))
            .await
            .unwrap();
        let matches: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!matches.is_empty(), "case-insensitive search should match");

        // Case-sensitive, wrong case -- should NOT match
        let result = client
            .call_tool_text(
                "search_adrs",
                json!({"query": "postgresql", "case_sensitive": true}),
            )
            .await
            .unwrap();
        let matches: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(
            matches.is_empty(),
            "case-sensitive search for 'postgresql' should not match 'PostgreSQL'"
        );

        // Case-sensitive, correct case -- should match
        let result = client
            .call_tool_text(
                "search_adrs",
                json!({"query": "PostgreSQL", "case_sensitive": true}),
            )
            .await
            .unwrap();
        let matches: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(
            !matches.is_empty(),
            "case-sensitive search for 'PostgreSQL' should match"
        );
    }

    #[tokio::test]
    async fn test_search_adrs_status_filter() {
        let (client, _tmp) = setup_client(false).await;

        // Init ADR is "Accepted", create a proposed one
        client
            .call_tool_text(
                "create_adr",
                json!({"title": "Proposed ADR about databases"}),
            )
            .await
            .unwrap();

        // Search with status=proposed -- should only find the proposed one
        let result = client
            .call_tool_text("search_adrs", json!({"query": "adr", "status": "proposed"}))
            .await
            .unwrap();
        let matches: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        for m in &matches {
            assert_eq!(
                m["status"].as_str().unwrap().to_lowercase(),
                "proposed",
                "all search results should be proposed"
            );
        }
    }

    #[tokio::test]
    async fn test_search_adrs_returns_snippets() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "Snippet Test ADR",
                    "context": "We need to evaluate the performance of the system under load.",
                    "decision": "Use a load balancer for better performance.",
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("search_adrs", json!({"query": "performance"}))
            .await
            .unwrap();
        let matches: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(
            !matches.is_empty(),
            "should find ADR with 'performance' in it"
        );

        let first = &matches[0];
        let snippets = first["matches"].as_array().unwrap();
        assert!(
            !snippets.is_empty(),
            "should have at least one match snippet"
        );
        for s in snippets {
            assert!(
                s["section"].is_string(),
                "each snippet should have a section"
            );
            assert!(
                s["snippet"].is_string(),
                "each snippet should have a snippet"
            );
        }
    }

    #[tokio::test]
    async fn test_run_doctor_healthy_repo() {
        let (client, _tmp) = setup_client(false).await;
        let result = client
            .call_tool_text("run_doctor", json!({}))
            .await
            .unwrap();

        let report: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(report["healthy"].is_boolean());
        assert!(report["issues"].is_array());
        assert_eq!(
            report["error_count"].as_u64().unwrap(),
            0,
            "fresh repo should have no errors"
        );
    }

    #[tokio::test]
    async fn test_export_adrs_all() {
        let (client, _tmp) = setup_client(false).await;
        let result = client
            .call_tool_text("export_adrs", json!({}))
            .await
            .unwrap();

        let export: serde_json::Value = serde_json::from_str(&result).unwrap();
        let adrs = export["adrs"].as_array().unwrap();
        assert!(
            !adrs.is_empty(),
            "export should contain at least the init ADR"
        );
        assert!(adrs[0]["number"].is_number());
        assert!(adrs[0]["title"].is_string());
    }

    #[tokio::test]
    async fn test_export_adrs_filtered_by_numbers() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text("create_adr", json!({"title": "ADR Two"}))
            .await
            .unwrap();
        client
            .call_tool_text("create_adr", json!({"title": "ADR Three"}))
            .await
            .unwrap();

        // Export only ADR #2
        let result = client
            .call_tool_text("export_adrs", json!({"numbers": [2]}))
            .await
            .unwrap();

        let export: serde_json::Value = serde_json::from_str(&result).unwrap();
        let adrs = export["adrs"].as_array().unwrap();
        assert_eq!(adrs.len(), 1, "should export exactly 1 ADR");
        assert_eq!(adrs[0]["number"].as_u64().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_export_adrs_metadata_only() {
        let (client, _tmp) = setup_client(false).await;

        client
            .call_tool_text(
                "create_adr",
                json!({
                    "title": "ADR with content",
                    "context": "Some context here.",
                    "decision": "Some decision here."
                }),
            )
            .await
            .unwrap();

        let result = client
            .call_tool_text("export_adrs", json!({"metadata_only": true}))
            .await
            .unwrap();

        let export: serde_json::Value = serde_json::from_str(&result).unwrap();
        let adrs = export["adrs"].as_array().unwrap();
        // Find ADR #2
        let adr2 = adrs
            .iter()
            .find(|a| a["number"].as_u64() == Some(2))
            .unwrap();
        // context, decision should be absent (null or not present) in metadata_only mode
        assert!(
            adr2.get("context").is_none() || adr2["context"].is_null(),
            "context should be absent in metadata_only mode"
        );
        assert!(
            adr2.get("decision").is_none() || adr2["decision"].is_null(),
            "decision should be absent in metadata_only mode"
        );
    }
}
