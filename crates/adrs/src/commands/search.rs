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

#[cfg(test)]
mod tests {
    use super::*;
    use adrs_core::AdrStatus;

    // ========== contains_match tests ==========

    #[test]
    fn test_contains_match_case_insensitive_found() {
        assert!(contains_match("Hello World", "world", false));
    }

    #[test]
    fn test_contains_match_case_insensitive_not_found() {
        assert!(!contains_match("Hello World", "rust", false));
    }

    #[test]
    fn test_contains_match_case_sensitive_found() {
        assert!(contains_match("Hello World", "World", true));
    }

    #[test]
    fn test_contains_match_case_sensitive_not_found() {
        assert!(!contains_match("Hello World", "world", true));
    }

    #[test]
    fn test_contains_match_empty_text() {
        assert!(!contains_match("", "query", false));
    }

    // ========== extract_snippet tests ==========

    #[test]
    fn test_extract_snippet_match_in_middle() {
        let text = "This is some context about the database decision and consequences.";
        let snippet = extract_snippet(text, "database", false);
        assert!(
            snippet.contains("database"),
            "Snippet should contain the match"
        );
    }

    #[test]
    fn test_extract_snippet_match_at_start() {
        let text = "database is the topic of this context section.";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("database"));
    }

    #[test]
    fn test_extract_snippet_match_at_end() {
        let text = "This context is about the database";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("database"));
    }

    #[test]
    fn test_extract_snippet_no_match_returns_first_chars() {
        // When no match found, returns up to 80 chars of text
        let text = "This context has no matching term at all.";
        let snippet = extract_snippet(text, "nonexistent", false);
        // Fallback returns the original text (it is under 80 chars)
        assert_eq!(snippet, text);
    }

    #[test]
    fn test_extract_snippet_long_text_no_match_truncates() {
        let text = "a".repeat(100);
        let snippet = extract_snippet(&text, "nonexistent", false);
        // Fallback truncates to 80 chars + "..."
        assert!(snippet.ends_with("..."));
        assert!(snippet.len() <= 83); // 80 chars + "..."
    }

    #[test]
    fn test_extract_snippet_case_insensitive() {
        let text = "The DATABASE decision was made.";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("DATABASE"));
    }

    #[test]
    fn test_extract_snippet_adds_ellipsis_for_truncated_start() {
        // Long text with match far from start should add leading ellipsis
        let prefix = "x ".repeat(25); // 50 chars
        let suffix = " y".repeat(25); // 50 chars
        let text = format!("{}match{}", prefix, suffix);
        let snippet = extract_snippet(&text, "match", false);
        assert!(snippet.contains("match"));
    }

    // ========== status_matches tests ==========

    #[test]
    fn test_status_matches_proposed_proposed() {
        assert!(status_matches(&AdrStatus::Proposed, &AdrStatus::Proposed));
    }

    #[test]
    fn test_status_matches_accepted_accepted() {
        assert!(status_matches(&AdrStatus::Accepted, &AdrStatus::Accepted));
    }

    #[test]
    fn test_status_matches_deprecated_deprecated() {
        assert!(status_matches(
            &AdrStatus::Deprecated,
            &AdrStatus::Deprecated
        ));
    }

    #[test]
    fn test_status_matches_superseded_superseded() {
        assert!(status_matches(
            &AdrStatus::Superseded,
            &AdrStatus::Superseded
        ));
    }

    #[test]
    fn test_status_matches_different_standard_statuses() {
        assert!(!status_matches(&AdrStatus::Proposed, &AdrStatus::Accepted));
        assert!(!status_matches(
            &AdrStatus::Accepted,
            &AdrStatus::Deprecated
        ));
    }

    #[test]
    fn test_status_matches_custom_custom_case_insensitive() {
        assert!(status_matches(
            &AdrStatus::Custom("Draft".into()),
            &AdrStatus::Custom("draft".into())
        ));
        assert!(status_matches(
            &AdrStatus::Custom("DRAFT".into()),
            &AdrStatus::Custom("Draft".into())
        ));
    }

    #[test]
    fn test_status_matches_custom_vs_standard_case_insensitive() {
        // "accepted" as Custom should match Accepted
        assert!(status_matches(
            &AdrStatus::Custom("accepted".into()),
            &AdrStatus::Accepted
        ));
        assert!(status_matches(
            &AdrStatus::Accepted,
            &AdrStatus::Custom("Accepted".into())
        ));
    }

    #[test]
    fn test_status_matches_custom_vs_standard_no_match() {
        assert!(!status_matches(
            &AdrStatus::Custom("draft".into()),
            &AdrStatus::Accepted
        ));
    }

    // ========== find_matches tests ==========

    #[test]
    fn test_find_matches_title_match() {
        let mut adr = Adr::new(1, "Use PostgreSQL for persistence");
        adr.context = "We need a database.".to_string();
        adr.decision = "We chose PostgreSQL.".to_string();
        adr.consequences = "Higher complexity.".to_string();

        let matches = find_matches(&adr, "postgresql", false, false);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.section == "Title"));
    }

    #[test]
    fn test_find_matches_no_match_returns_empty() {
        let mut adr = Adr::new(1, "Use PostgreSQL");
        adr.context = "We need a database.".to_string();
        adr.decision = "We chose PostgreSQL.".to_string();
        adr.consequences = "Higher complexity.".to_string();

        let matches = find_matches(&adr, "nonexistent_term_xyz", false, false);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_matches_title_only_mode() {
        let mut adr = Adr::new(1, "Use PostgreSQL");
        adr.context = "This context also mentions PostgreSQL.".to_string();

        // In title_only mode, only the title is checked
        let matches = find_matches(&adr, "postgresql", true, false);
        assert!(matches.iter().any(|m| m.section == "Title"));
        // Should NOT search context in title_only mode
        assert!(!matches.iter().any(|m| m.section == "Context"));
    }

    #[test]
    fn test_find_matches_context_match() {
        let mut adr = Adr::new(1, "Architecture Decision");
        adr.context = "We need to store data in a relational database.".to_string();
        adr.decision = "We will use PostgreSQL.".to_string();
        adr.consequences = "Team needs training.".to_string();

        let matches = find_matches(&adr, "relational", false, false);
        assert!(matches.iter().any(|m| m.section == "Context"));
    }

    #[test]
    fn test_find_matches_multiple_sections() {
        let mut adr = Adr::new(1, "test adr");
        adr.context = "The keyword is here.".to_string();
        adr.decision = "The keyword applies.".to_string();
        adr.consequences = "keyword consequences.".to_string();

        let matches = find_matches(&adr, "keyword", false, false);
        // Should find in context, decision, consequences
        assert!(matches.len() >= 3);
    }
}
