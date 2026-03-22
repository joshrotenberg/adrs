//! MCP (Model Context Protocol) server for ADR integration.
//!
//! This module provides an MCP server that allows AI agents to interact with ADRs.
//! Enable with the `mcp` feature flag.

use adrs_core::{AdrStatus, LinkKind, Repository};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
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

    // Write tools
    let create_adr = adr_tool!(
        rw,
        state,
        "create_adr",
        "Create a new Architecture Decision Record. The ADR will be created with 'proposed' status and requires human review before acceptance. Returns the created ADR details including its number and file path.",
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
        // Write tools
        .tool(create_adr)
        .tool(update_status)
        .tool(link_adrs)
        .tool(update_content)
        .tool(update_tags)
        .tool(bulk_update_status)
}

// Business logic implementations (unchanged).

impl AdrState {
    fn list_adrs_impl(&self, params: ListAdrsParams) -> Result<String, String> {
        let repo = self.open_repo()?;
        let adrs = repo.list().map_err(|e| e.to_string())?;

        let mut summaries: Vec<AdrSummary> = adrs
            .iter()
            .map(|adr| AdrSummary {
                number: adr.number,
                title: adr.title.clone(),
                status: adr.status.to_string(),
                date: Some(adr.date.to_string()),
                tags: adr.tags.clone(),
            })
            .collect();

        // Apply filters
        if let Some(ref status) = params.status {
            let status_lower = status.to_lowercase();
            summaries.retain(|s| s.status.to_lowercase() == status_lower);
        }

        if let Some(ref tag) = params.tag {
            let tag_lower = tag.to_lowercase();
            summaries.retain(|s| s.tags.iter().any(|t| t.to_lowercase() == tag_lower));
        }

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

        let query_lower = params.query.to_lowercase();
        let title_only = params.title_only.unwrap_or(false);

        let mut matches: Vec<AdrSummary> = Vec::new();

        for adr in &adrs {
            let title_match = adr.title.to_lowercase().contains(&query_lower);

            let content_match = if !title_only {
                repo.read_content(adr)
                    .map(|c| c.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            } else {
                false
            };

            if title_match || content_match {
                matches.push(AdrSummary {
                    number: adr.number,
                    title: adr.title.clone(),
                    status: adr.status.to_string(),
                    date: Some(adr.date.to_string()),
                    tags: adr.tags.clone(),
                });
            }
        }

        serde_json::to_string_pretty(&matches).map_err(|e| e.to_string())
    }

    fn create_adr_impl(&self, params: CreateAdrParams) -> Result<String, String> {
        let repo = self.open_repo()?;

        let (mut adr, path) = if let Some(supersedes) = params.supersedes {
            repo.supersede(&params.title, supersedes)
                .map_err(|e| e.to_string())?
        } else {
            repo.new_adr(&params.title).map_err(|e| e.to_string())?
        };

        // Update content if provided
        let mut content_updated = false;
        if let Some(context) = params.context {
            adr.context = context;
            content_updated = true;
        }
        if let Some(decision) = params.decision {
            adr.decision = decision;
            content_updated = true;
        }
        if let Some(consequences) = params.consequences {
            adr.consequences = consequences;
            content_updated = true;
        }

        // Re-render if content was provided
        let final_path = if content_updated {
            repo.update(&adr).map_err(|e| e.to_string())?
        } else {
            path
        };

        #[derive(Serialize)]
        struct CreateResponse {
            message: String,
            number: u32,
            title: String,
            status: String,
            path: String,
            content_populated: bool,
        }

        let response = CreateResponse {
            message: "ADR created successfully. Please review and edit as needed.".to_string(),
            number: adr.number,
            title: adr.title,
            status: adr.status.to_string(),
            path: final_path.display().to_string(),
            content_populated: content_updated,
        };

        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    }

    fn update_status_impl(&self, params: UpdateStatusParams) -> Result<String, String> {
        let repo = self.open_repo()?;

        let status: AdrStatus = params.status.parse().unwrap(); // Infallible

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

        // Update only provided fields
        if let Some(context) = params.context {
            adr.context = context;
        }
        if let Some(decision) = params.decision {
            adr.decision = decision;
        }
        if let Some(consequences) = params.consequences {
            adr.consequences = consequences;
        }

        let path = repo.update(&adr).map_err(|e| e.to_string())?;

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
        let info = client.server_info().unwrap();
        assert_eq!(info.server_info.name, "adrs");
        assert!(info.capabilities.tools.is_some());
    }

    #[tokio::test]
    async fn test_list_tools_returns_all_15() {
        let (client, _tmp) = setup_client(false).await;
        let tools = client.list_all_tools().await.unwrap();
        assert_eq!(tools.len(), 15, "expected 15 tools, got {}", tools.len());

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
    async fn test_ping() {
        let (client, _tmp) = setup_client(false).await;
        client.ping().await.unwrap();
    }
}
