//! MCP (Model Context Protocol) server for ADR integration.
//!
//! This module provides an MCP server that allows AI agents to interact with ADRs.
//! Enable with the `mcp` feature flag.

use adrs_core::Repository;
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
}

impl ServerHandler for AdrService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "ADR (Architecture Decision Record) management server. \
                Use list_adrs to see all decisions, get_adr to read a specific one, \
                and search_adrs to find relevant decisions. \
                ADRs document important architectural decisions and their context."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Run the MCP server on stdio.
pub async fn serve(root: PathBuf) -> Result<()> {
    use rmcp::ServiceExt;
    use rmcp::transport::stdio;

    let service = AdrService::new(root);
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
