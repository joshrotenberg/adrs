//! MCP (Model Context Protocol) server for ADR integration.
//!
//! This module provides an MCP server that allows AI agents to interact with ADRs.
//! Enable with the `mcp` feature flag.

use adrs_core::{AdrStatus, LinkKind, Repository};
use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ADR MCP Service
#[derive(Debug, Clone)]
pub struct AdrService {
    /// Root directory for the ADR repository
    root: PathBuf,
    /// Tool router for MCP tools (used by macro-generated code)
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl AdrService {
    /// Create a new ADR service for the given root directory.
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            tool_router: Self::tool_router(),
        }
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
pub struct GetAdrSectionsParams {
    /// ADR number to retrieve
    #[schemars(description = "The ADR number (e.g., 1, 2, 3)")]
    pub number: u32,
}

// Response types

#[derive(Debug, Serialize)]
struct AdrSummary {
    number: u32,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AdrDetail {
    number: u32,
    title: String,
    status: String,
    date: Option<String>,
    tags: Vec<String>,
    content: String,
    links: Vec<LinkInfo>,
}

#[derive(Debug, Serialize)]
struct LinkInfo {
    kind: String,
    target: u32,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[tool_router]
impl AdrService {
    /// List all ADRs in the repository
    #[tool(
        description = "List all Architecture Decision Records. Returns summary information for each ADR including number, title, status, and date. Optionally filter by status or tag."
    )]
    fn list_adrs(&self, Parameters(params): Parameters<ListAdrsParams>) -> String {
        match self.list_adrs_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get a specific ADR by number
    #[tool(
        description = "Get the full content of an Architecture Decision Record by its number. Returns the complete ADR including title, status, content, and links."
    )]
    fn get_adr(&self, Parameters(params): Parameters<GetAdrParams>) -> String {
        match self.get_adr_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Search ADRs for matching content
    #[tool(
        description = "Search Architecture Decision Records for matching text. Searches both titles and content by default. Use title_only=true to search only titles."
    )]
    fn search_adrs(&self, Parameters(params): Parameters<SearchAdrsParams>) -> String {
        match self.search_adrs_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    // Write operations

    /// Create a new ADR
    #[tool(
        description = "Create a new Architecture Decision Record. The ADR will be created with 'proposed' status and requires human review before acceptance. Returns the created ADR details including its number and file path."
    )]
    fn create_adr(&self, Parameters(params): Parameters<CreateAdrParams>) -> String {
        match self.create_adr_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update an ADR's status
    #[tool(
        description = "Update the status of an existing ADR. Valid statuses: proposed, accepted, deprecated, superseded, rejected. For 'superseded', provide the superseded_by number. Note: Status changes should be reviewed by humans."
    )]
    fn update_status(&self, Parameters(params): Parameters<UpdateStatusParams>) -> String {
        match self.update_status_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Link two ADRs together
    #[tool(
        description = "Create a bidirectional link between two ADRs. Link types: 'Supersedes', 'Amends', or 'Relates to'. The reverse link is automatically created on the target ADR."
    )]
    fn link_adrs(&self, Parameters(params): Parameters<LinkAdrsParams>) -> String {
        match self.link_adrs_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update ADR content sections
    #[tool(
        description = "Update the content sections (context, decision, consequences) of an existing ADR. Only provided fields are updated; omitted fields are preserved. Changes should be reviewed by humans."
    )]
    fn update_content(&self, Parameters(params): Parameters<UpdateContentParams>) -> String {
        match self.update_content_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update ADR tags
    #[tool(
        description = "Add or replace tags on an ADR. Requires NextGen mode (YAML frontmatter). Use replace=true to replace all tags, or false/omit to append."
    )]
    fn update_tags(&self, Parameters(params): Parameters<UpdateTagsParams>) -> String {
        match self.update_tags_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get repository information
    #[tool(
        description = "Get information about the ADR repository including mode (compatible/nextgen), ADR count, and configuration."
    )]
    fn get_repository_info(&self) -> String {
        match self.get_repository_info_impl() {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get ADRs related to a specific ADR
    #[tool(
        description = "Get all ADRs that are linked to or from a specific ADR. Returns both incoming and outgoing links with their types."
    )]
    fn get_related_adrs(&self, Parameters(params): Parameters<GetRelatedParams>) -> String {
        match self.get_related_adrs_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get ADR with parsed sections
    #[tool(
        description = "Get an ADR with its content parsed into separate sections (context, decision, consequences). Returns structured data instead of raw markdown, making it easier to analyze specific sections independently."
    )]
    fn get_adr_sections(&self, Parameters(params): Parameters<GetAdrSectionsParams>) -> String {
        match self.get_adr_sections_impl(params) {
            Ok(json) => json,
            Err(e) => format!("Error: {}", e),
        }
    }
}

impl AdrService {
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

    // Phase 2: Write operations

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

        let path = repo.update(&adr).map_err(|e| e.to_string())?;

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
}

impl ServerHandler for AdrService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "ADR (Architecture Decision Record) management server. \
                Use list_adrs to see all decisions, get_adr to read a specific one, \
                and search_adrs to find relevant decisions. For modifications: create_adr creates new ADRs \
                (always as 'proposed' status for human review), update_status changes an ADR's status, \
                link_adrs creates bidirectional links, update_content edits ADR sections, \
                and update_tags manages ADR tags (NextGen mode only). \
                Use get_repository_info to understand the repo configuration and get_related_adrs \
                to explore ADR relationships. ADRs document important architectural decisions and their context."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Run the MCP server on stdio.
pub async fn serve_stdio(root: PathBuf) -> Result<()> {
    use rmcp::ServiceExt;
    use rmcp::transport::stdio;

    let service = AdrService::new(root);
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}

/// Run the MCP server over HTTP.
#[cfg(feature = "mcp-http")]
pub async fn serve_http(root: PathBuf, addr: std::net::SocketAddr) -> Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };
    use tokio_util::sync::CancellationToken;

    let ct = CancellationToken::new();

    let service: StreamableHttpService<AdrService, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(AdrService::new(root.clone())),
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: Some(std::time::Duration::from_secs(30)),
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(addr).await?;

    eprintln!("MCP server listening on http://{}/mcp", addr);

    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            eprintln!("\nShutting down MCP server...");
            ct.cancel();
        })
        .await?;

    Ok(())
}
