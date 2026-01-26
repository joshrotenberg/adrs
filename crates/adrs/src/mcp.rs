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
#[allow(dead_code)] // Fields exposed in schema for future content population
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

        let (adr, path) = if let Some(supersedes) = params.supersedes {
            repo.supersede(&params.title, supersedes)
                .map_err(|e| e.to_string())?
        } else {
            repo.new_adr(&params.title).map_err(|e| e.to_string())?
        };

        // Note: Created ADRs are always in 'proposed' status by default
        // The optional context/decision/consequences fields are noted for future enhancement
        // when we support updating ADR content programmatically

        #[derive(Serialize)]
        struct CreateResponse {
            message: String,
            number: u32,
            title: String,
            status: String,
            path: String,
        }

        let response = CreateResponse {
            message: "ADR created successfully. Please review and edit as needed.".to_string(),
            number: adr.number,
            title: adr.title,
            status: adr.status.to_string(),
            path: path.display().to_string(),
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
}

impl ServerHandler for AdrService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "ADR (Architecture Decision Record) management server. \
                Use list_adrs to see all decisions, get_adr to read a specific one, \
                and search_adrs to find relevant decisions. \
                For modifications: create_adr creates new ADRs (always as 'proposed' status for human review), \
                update_status changes an ADR's status, and link_adrs creates bidirectional links. \
                ADRs document important architectural decisions and their context."
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
