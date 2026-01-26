//! Search ADRs command.

use adrs_core::{Adr, AdrStatus, Repository};
use anyhow::{Context, Result};
use std::path::Path;

/// Search ADRs for matching content.
pub fn search(
    root: &Path,
    query: &str,
    title_only: bool,
    status_filter: Option<String>,
    case_sensitive: bool,
) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let adrs = repo.list()?;

    // Parse status filter
    let status_filter: Option<AdrStatus> = status_filter.map(|s| s.parse().unwrap());

    // Prepare query for matching
    let query_normalized = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    let mut found_any = false;

    for adr in adrs {
        // Apply status filter
        if let Some(ref filter_status) = status_filter
            && !status_matches(&adr.status, filter_status)
        {
            continue;
        }

        // Search for matches
        let matches = find_matches(&adr, &query_normalized, title_only, case_sensitive);

        if !matches.is_empty() {
            found_any = true;
            print_result(&adr, &matches, query);
        }
    }

    if !found_any {
        println!("No matches found for '{}'", query);
    }

    Ok(())
}

/// Find matches in an ADR.
fn find_matches(
    adr: &Adr,
    query: &str,
    title_only: bool,
    case_sensitive: bool,
) -> Vec<SearchMatch> {
    let mut matches = Vec::new();

    // Check title
    if contains_match(&adr.title, query, case_sensitive) {
        matches.push(SearchMatch {
            section: "Title".to_string(),
            snippet: adr.title.clone(),
        });
    }

    if title_only {
        return matches;
    }

    // Check context
    if contains_match(&adr.context, query, case_sensitive) {
        matches.push(SearchMatch {
            section: "Context".to_string(),
            snippet: extract_snippet(&adr.context, query, case_sensitive),
        });
    }

    // Check decision
    if contains_match(&adr.decision, query, case_sensitive) {
        matches.push(SearchMatch {
            section: "Decision".to_string(),
            snippet: extract_snippet(&adr.decision, query, case_sensitive),
        });
    }

    // Check consequences
    if contains_match(&adr.consequences, query, case_sensitive) {
        matches.push(SearchMatch {
            section: "Consequences".to_string(),
            snippet: extract_snippet(&adr.consequences, query, case_sensitive),
        });
    }

    matches
}

/// Check if text contains the query.
fn contains_match(text: &str, query: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        text.contains(query)
    } else {
        text.to_lowercase().contains(query)
    }
}

/// Extract a snippet around the match.
fn extract_snippet(text: &str, query: &str, case_sensitive: bool) -> String {
    let text_search = if case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    };

    if let Some(pos) = text_search.find(query) {
        // Get some context around the match
        let start = pos.saturating_sub(40);
        let end = (pos + query.len() + 40).min(text.len());

        // Find word boundaries
        let start = text[..start]
            .rfind(char::is_whitespace)
            .map(|p| p + 1)
            .unwrap_or(start);
        let end = text[end..]
            .find(char::is_whitespace)
            .map(|p| end + p)
            .unwrap_or(end);

        let mut snippet = text[start..end].to_string();

        // Add ellipsis if truncated
        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < text.len() {
            snippet = format!("{}...", snippet);
        }

        // Replace newlines with spaces for cleaner output
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

/// Print a search result.
fn print_result(adr: &Adr, matches: &[SearchMatch], _query: &str) {
    println!("{}. {}", adr.number, adr.title);

    for m in matches {
        if m.section != "Title" {
            println!("   {}: {}", m.section, m.snippet);
        }
    }

    println!();
}

/// A match found in an ADR.
struct SearchMatch {
    section: String,
    snippet: String,
}

/// Check if two statuses match (case-insensitive).
fn status_matches(adr_status: &AdrStatus, filter_status: &AdrStatus) -> bool {
    match (adr_status, filter_status) {
        (AdrStatus::Proposed, AdrStatus::Proposed) => true,
        (AdrStatus::Accepted, AdrStatus::Accepted) => true,
        (AdrStatus::Deprecated, AdrStatus::Deprecated) => true,
        (AdrStatus::Superseded, AdrStatus::Superseded) => true,
        (AdrStatus::Custom(a), AdrStatus::Custom(b)) => a.to_lowercase() == b.to_lowercase(),
        (AdrStatus::Custom(s), standard) | (standard, AdrStatus::Custom(s)) => {
            s.to_lowercase() == standard.to_string().to_lowercase()
        }
        _ => false,
    }
}
