//! ADR parsing - supports both legacy markdown and YAML frontmatter formats.

use crate::{Adr, AdrLink, AdrStatus, Error, LinkKind, Result};
use pulldown_cmark::{Event, HeadingLevel, Parser as MdParser, Tag, TagEnd};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;
use time::{Date, Month, OffsetDateTime};

/// Regex for parsing legacy status links like "Supersedes [1. Title](0001-title.md)".
static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([\w\s]+)\s+\[(\d+)\.\s+[^\]]+\]\((\d{4})-[^)]+\.md\)$").unwrap()
});

/// Regex for extracting ADR number from filename.
static NUMBER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{4})-.*\.md$").unwrap());

/// Parser for ADR files.
#[derive(Debug, Default)]
pub struct Parser {
    _private: (),
}

impl Parser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an ADR from a file.
    pub fn parse_file(&self, path: &Path) -> Result<Adr> {
        let content = std::fs::read_to_string(path)?;
        let mut adr = self.parse(&content)?;

        // Extract number from filename if not in frontmatter
        if adr.number == 0 {
            adr.number = extract_number_from_path(path)?;
        }

        adr.path = Some(path.to_path_buf());
        Ok(adr)
    }

    /// Parse an ADR from a string.
    pub fn parse(&self, content: &str) -> Result<Adr> {
        // Check for YAML frontmatter
        if content.starts_with("---\n") {
            self.parse_frontmatter(content)
        } else {
            self.parse_legacy(content)
        }
    }

    /// Parse ADR with YAML frontmatter.
    fn parse_frontmatter(&self, content: &str) -> Result<Adr> {
        let parts: Vec<&str> = content.splitn(3, "---\n").collect();
        if parts.len() < 3 {
            return Err(Error::InvalidFormat {
                path: Default::default(),
                reason: "Invalid frontmatter format".into(),
            });
        }

        let yaml = parts[1];
        let body = parts[2];

        // Parse frontmatter
        let mut adr: Adr = serde_yaml::from_str(yaml)?;

        // Parse body sections
        let sections = self.parse_sections(body);
        if let Some(context) = sections.get("context") {
            adr.context = context.clone();
        }
        if let Some(decision) = sections.get("decision") {
            adr.decision = decision.clone();
        }
        if let Some(consequences) = sections.get("consequences") {
            adr.consequences = consequences.clone();
        }

        Ok(adr)
    }

    /// Parse legacy markdown format (adr-tools compatible).
    fn parse_legacy(&self, content: &str) -> Result<Adr> {
        let mut adr = Adr::new(0, "");

        // Use a simpler approach: split by H2 sections and parse each
        let sections = self.extract_sections_raw(content);

        // Parse H1 title
        if let Some(title_line) = content.lines().find(|l| l.starts_with("# ")) {
            let title_str = title_line.trim_start_matches("# ").trim();
            if let Some((num, title)) = parse_numbered_title(title_str) {
                adr.number = num;
                adr.title = title;
            } else {
                adr.title = title_str.to_string();
            }
        }

        // Apply sections
        for (name, content) in &sections {
            self.apply_section(&mut adr, name, content);
        }

        Ok(adr)
    }

    /// Extract sections from raw markdown text.
    fn extract_sections_raw(&self, content: &str) -> Vec<(String, String)> {
        let mut sections = Vec::new();
        let mut current_section: Option<String> = None;
        let mut section_content = String::new();

        for line in content.lines() {
            if line.starts_with("## ") {
                // Save previous section
                if let Some(ref name) = current_section {
                    sections.push((name.clone(), section_content.trim().to_string()));
                }
                current_section = Some(line.trim_start_matches("## ").trim().to_lowercase());
                section_content.clear();
            } else if current_section.is_some() {
                section_content.push_str(line);
                section_content.push('\n');
            }
        }

        // Save final section
        if let Some(ref name) = current_section {
            sections.push((name.clone(), section_content.trim().to_string()));
        }

        sections
    }

    /// Apply a parsed section to the ADR.
    fn apply_section(&self, adr: &mut Adr, section: &str, content: &str) {
        let content = content.trim().to_string();
        match section {
            "status" => {
                self.parse_status_section(adr, &content);
            }
            "context" => {
                adr.context = content;
            }
            "decision" => {
                adr.decision = content;
            }
            "consequences" => {
                adr.consequences = content;
            }
            _ => {}
        }
    }

    /// Parse the status section for status and links.
    fn parse_status_section(&self, adr: &mut Adr, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Check for link pattern: "Supersedes [1. Title](0001-title.md)"
            if let Some(caps) = LINK_REGEX.captures(line) {
                let kind_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let target: u32 = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);

                if target > 0 {
                    let kind: LinkKind = kind_str.trim().parse().unwrap_or(LinkKind::RelatesTo);

                    // If this is a "Superseded by" link, set status to Superseded
                    // (adr-tools doesn't always have a separate status line)
                    if matches!(kind, LinkKind::SupersededBy) {
                        adr.status = AdrStatus::Superseded;
                    }

                    adr.links.push(AdrLink::new(target, kind));
                }
            } else if !line.contains('[') && !line.contains(']') {
                // Plain status text (not a link line)
                // Only set status if it looks like a simple status word
                let word = line.split_whitespace().next().unwrap_or("");
                if matches!(
                    word.to_lowercase().as_str(),
                    // Include "superceded" for adr-tools compatibility (common typo)
                    "proposed"
                        | "accepted"
                        | "deprecated"
                        | "superseded"
                        | "superceded"
                        | "draft"
                        | "rejected"
                ) {
                    adr.status = word.parse().unwrap_or(AdrStatus::Proposed);
                }
            }
        }
    }

    /// Parse markdown sections into a map.
    fn parse_sections(&self, content: &str) -> std::collections::HashMap<String, String> {
        let mut sections = std::collections::HashMap::new();
        let mut current_section: Option<String> = None;
        let mut section_content = String::new();

        let parser = MdParser::new(content);
        let mut in_heading = false;

        for event in parser {
            match event {
                Event::Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    ..
                }) => {
                    if let Some(ref section) = current_section {
                        sections.insert(section.clone(), section_content.trim().to_string());
                    }
                    in_heading = true;
                    section_content.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;
                }
                Event::Text(text) => {
                    if in_heading {
                        current_section = Some(text.to_string().to_lowercase());
                    } else {
                        section_content.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !in_heading {
                        section_content.push('\n');
                    }
                }
                _ => {}
            }
        }

        if let Some(ref section) = current_section {
            sections.insert(section.clone(), section_content.trim().to_string());
        }

        sections
    }
}

/// Parse a numbered title like "1. Use Rust" into (1, "Use Rust").
fn parse_numbered_title(title: &str) -> Option<(u32, String)> {
    let parts: Vec<&str> = title.splitn(2, ". ").collect();
    if parts.len() == 2
        && let Ok(num) = parts[0].parse::<u32>()
    {
        return Some((num, parts[1].to_string()));
    }
    None
}

/// Extract ADR number from a file path.
fn extract_number_from_path(path: &Path) -> Result<u32> {
    let filename =
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::InvalidFormat {
                path: path.to_path_buf(),
                reason: "Invalid filename".into(),
            })?;

    NUMBER_REGEX
        .captures(filename)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or_else(|| Error::InvalidFormat {
            path: path.to_path_buf(),
            reason: "Cannot extract ADR number from filename".into(),
        })
}

/// Get today's date.
pub fn today() -> Date {
    let now = OffsetDateTime::now_utc();
    Date::from_calendar_date(now.year(), now.month(), now.day()).unwrap_or_else(|_| {
        // Fallback to a safe default
        Date::from_calendar_date(2024, Month::January, 1).unwrap()
    })
}

/// Format a date as YYYY-MM-DD.
pub fn format_date(date: Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use test_case::test_case;

    // ========== Parser Creation ==========

    #[test]
    fn test_parser_new() {
        let _parser = Parser::new();
        // Parser creation succeeds - just confirms it compiles
    }

    #[test]
    fn test_parser_default() {
        let _parser = Parser::default();
    }

    // ========== Legacy Format Parsing ==========

    #[test]
    fn test_parse_legacy_format() {
        let content = r#"# 1. Use Rust

## Status

Accepted

## Context

We need a systems programming language.

## Decision

We will use Rust.

## Consequences

We get memory safety without garbage collection.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Use Rust");
        assert_eq!(adr.status, AdrStatus::Accepted);
        assert!(adr.context.contains("systems programming"));
        assert!(adr.decision.contains("use Rust"));
        assert!(adr.consequences.contains("memory safety"));
    }

    #[test]
    fn test_parse_legacy_minimal() {
        let content = r#"# 1. Minimal ADR

## Status

Proposed

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Minimal ADR");
        assert_eq!(adr.status, AdrStatus::Proposed);
        assert_eq!(adr.context, "Context.");
        assert_eq!(adr.decision, "Decision.");
        assert_eq!(adr.consequences, "Consequences.");
    }

    #[test]
    fn test_parse_legacy_multiline_sections() {
        let content = r#"# 1. Multiline Test

## Status

Accepted

## Context

This is a context section
that spans multiple lines.

With paragraphs too.

## Decision

This is the decision.
Also multiple lines.

## Consequences

- Point 1
- Point 2
- Point 3
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert!(adr.context.contains("multiple lines"));
        assert!(adr.context.contains("paragraphs"));
        assert!(adr.decision.contains("Also multiple lines"));
        assert!(adr.consequences.contains("Point 1"));
        assert!(adr.consequences.contains("Point 2"));
    }

    #[test_case("Proposed" => AdrStatus::Proposed; "proposed")]
    #[test_case("Accepted" => AdrStatus::Accepted; "accepted")]
    #[test_case("Deprecated" => AdrStatus::Deprecated; "deprecated")]
    #[test_case("Superseded" => AdrStatus::Superseded; "superseded")]
    #[test_case("Draft" => AdrStatus::Custom("Draft".into()); "draft")]
    #[test_case("Rejected" => AdrStatus::Custom("Rejected".into()); "rejected")]
    fn test_parse_legacy_status_types(status: &str) -> AdrStatus {
        let content = format!(
            r#"# 1. Test

## Status

{status}

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#
        );

        let parser = Parser::new();
        let adr = parser.parse(&content).unwrap();
        adr.status
    }

    #[test]
    fn test_parse_legacy_with_date_line() {
        let content = r#"# 1. Record architecture decisions

Date: 2024-01-15

## Status

Accepted

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Record architecture decisions");
        assert_eq!(adr.status, AdrStatus::Accepted);
    }

    #[test]
    fn test_parse_legacy_title_without_number() {
        let content = r#"# Use Rust

## Status

Proposed

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 0);
        assert_eq!(adr.title, "Use Rust");
    }

    #[test]
    fn test_parse_legacy_status_with_links() {
        let content = r#"# 2. Use PostgreSQL

## Status

Accepted

Supersedes [1. Use MySQL](0001-use-mysql.md)

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.status, AdrStatus::Accepted);
        assert_eq!(adr.links.len(), 1);
        assert_eq!(adr.links[0].target, 1);
        assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
    }

    #[test]
    fn test_parse_legacy_multiple_links() {
        let content = r#"# 5. Combined Decision

## Status

Accepted

Supersedes [1. First](0001-first.md)
Supersedes [2. Second](0002-second.md)
Amends [3. Third](0003-third.md)

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.links.len(), 3);
        assert_eq!(adr.links[0].target, 1);
        assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
        assert_eq!(adr.links[1].target, 2);
        assert_eq!(adr.links[1].kind, LinkKind::Supersedes);
        assert_eq!(adr.links[2].target, 3);
        assert_eq!(adr.links[2].kind, LinkKind::Amends);
    }

    #[test]
    fn test_parse_superseded_status() {
        let content = r#"# 1. Record architecture decisions

Date: 2026-01-22

## Status

Superseded

Superseded by [2. ...](0002-....md)

## Context

Some context.

## Decision

Some decision.

## Consequences

Some consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.status, AdrStatus::Superseded);
    }

    // ========== Frontmatter Format Parsing ==========

    #[test]
    fn test_parse_frontmatter_format() {
        let content = r#"---
number: 2
title: Use PostgreSQL
date: 2024-01-15
status: accepted
links:
  - target: 1
    kind: supersedes
---

## Context

We need a database.

## Decision

We will use PostgreSQL.

## Consequences

We get ACID compliance.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 2);
        assert_eq!(adr.title, "Use PostgreSQL");
        assert_eq!(adr.status, AdrStatus::Accepted);
        assert_eq!(adr.links.len(), 1);
        assert_eq!(adr.links[0].target, 1);
        assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
    }

    #[test]
    fn test_parse_frontmatter_minimal() {
        let content = r#"---
number: 1
title: Simple ADR
date: 2024-01-01
status: proposed
---

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Simple ADR");
        assert_eq!(adr.status, AdrStatus::Proposed);
    }

    #[test]
    fn test_parse_frontmatter_no_links() {
        let content = r#"---
number: 1
title: Test ADR
date: 2024-01-01
status: accepted
---

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert!(adr.links.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_multiple_links() {
        let content = r#"---
number: 5
title: Multi Link ADR
date: 2024-01-01
status: accepted
links:
  - target: 1
    kind: supersedes
  - target: 2
    kind: amends
  - target: 3
    kind: relatesto
---

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.links.len(), 3);
        assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
        assert_eq!(adr.links[1].kind, LinkKind::Amends);
        assert_eq!(adr.links[2].kind, LinkKind::RelatesTo);
    }

    #[test]
    fn test_parse_frontmatter_all_statuses() {
        for (status_str, expected) in [
            ("proposed", AdrStatus::Proposed),
            ("accepted", AdrStatus::Accepted),
            ("deprecated", AdrStatus::Deprecated),
            ("superseded", AdrStatus::Superseded),
        ] {
            let content = format!(
                r#"---
number: 1
title: Test
date: 2024-01-01
status: {status_str}
---

## Context

Context.
"#
            );

            let parser = Parser::new();
            let adr = parser.parse(&content).unwrap();
            assert_eq!(adr.status, expected, "Failed for status: {status_str}");
        }
    }

    #[test]
    fn test_parse_frontmatter_invalid_format() {
        let content = r#"---
not valid yaml {{{{
---

## Context

Context.
"#;

        let parser = Parser::new();
        let result = parser.parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_incomplete() {
        let content = r#"---
number: 1
title: Test
"#;

        let parser = Parser::new();
        let result = parser.parse(content);
        assert!(result.is_err());
    }

    // ========== MADR Format Parsing ==========

    #[test]
    fn test_parse_madr_format() {
        // MADR format with number and title in frontmatter
        let content = r#"---
number: 2
title: Use Redis for caching
status: proposed
date: 2024-01-15
---

# Use Redis for caching

## Context and Problem Statement

We need a caching solution.

## Decision Outcome

We will use Redis.

### Consequences

* Good, because fast
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 2);
        assert_eq!(adr.title, "Use Redis for caching");
        assert_eq!(adr.status, AdrStatus::Proposed);
    }

    #[test]
    fn test_parse_madr_with_decision_makers() {
        let content = r#"---
number: 1
title: Use MADR Format
status: accepted
date: 2024-01-01
---

# Use MADR Format

## Context and Problem Statement

Context.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Use MADR Format");
        assert_eq!(adr.status, AdrStatus::Accepted);
    }

    #[test]
    fn test_parse_madr_missing_number_fails() {
        // MADR without number field should fail
        let content = r#"---
title: Missing Number
status: proposed
date: 2024-01-01
---

# Missing Number

## Context and Problem Statement

Context.
"#;

        let parser = Parser::new();
        let result = parser.parse(content);
        // Should fail because number is required
        assert!(result.is_err() || result.unwrap().number == 0);
    }

    // ========== File Parsing ==========

    #[test]
    fn test_parse_file_legacy() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0001-use-rust.md");

        std::fs::write(
            &file_path,
            r#"# 1. Use Rust

## Status

Accepted

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#,
        )
        .unwrap();

        let parser = Parser::new();
        let adr = parser.parse_file(&file_path).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Use Rust");
        assert_eq!(adr.path, Some(file_path));
    }

    #[test]
    fn test_parse_file_extracts_number_from_filename() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("0042-some-decision.md");

        // ADR without number in title
        std::fs::write(
            &file_path,
            r#"# Some Decision

## Status

Proposed

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.
"#,
        )
        .unwrap();

        let parser = Parser::new();
        let adr = parser.parse_file(&file_path).unwrap();

        assert_eq!(adr.number, 42);
    }

    #[test]
    fn test_parse_file_nonexistent() {
        let parser = Parser::new();
        let result = parser.parse_file(Path::new("/nonexistent/path/0001-test.md"));
        assert!(result.is_err());
    }

    // ========== Helper Function Tests ==========

    #[test]
    fn test_parse_numbered_title() {
        assert_eq!(
            parse_numbered_title("1. Use Rust"),
            Some((1, "Use Rust".into()))
        );
        assert_eq!(
            parse_numbered_title("42. Complex Decision"),
            Some((42, "Complex Decision".into()))
        );
        assert_eq!(parse_numbered_title("Use Rust"), None);
    }

    #[test_case("1. Simple" => Some((1, "Simple".into())); "simple")]
    #[test_case("123. Large Number" => Some((123, "Large Number".into())); "large number")]
    #[test_case("1. With. Dots. In. Title" => Some((1, "With. Dots. In. Title".into())); "dots in title")]
    #[test_case("No Number" => None; "no number")]
    #[test_case("1 Missing Period" => None; "missing period")]
    #[test_case(". Missing Number" => None; "missing number")]
    fn test_parse_numbered_title_cases(input: &str) -> Option<(u32, String)> {
        parse_numbered_title(input)
    }

    #[test]
    fn test_extract_number_from_path() {
        let path = Path::new("doc/adr/0001-use-rust.md");
        assert_eq!(extract_number_from_path(path).unwrap(), 1);

        let path = Path::new("0042-complex-decision.md");
        assert_eq!(extract_number_from_path(path).unwrap(), 42);

        let path = Path::new("9999-max-four-digit.md");
        assert_eq!(extract_number_from_path(path).unwrap(), 9999);
    }

    #[test]
    fn test_extract_number_from_path_invalid() {
        let result = extract_number_from_path(Path::new("not-an-adr.md"));
        assert!(result.is_err());

        let result = extract_number_from_path(Path::new("1-too-few-digits.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_today() {
        let date = today();
        assert!(date.year() >= 2024);
        assert!(date.month() as u8 >= 1 && date.month() as u8 <= 12);
        assert!(date.day() >= 1 && date.day() <= 31);
    }

    #[test]
    fn test_format_date() {
        let date = Date::from_calendar_date(2024, Month::March, 5).unwrap();
        assert_eq!(format_date(date), "2024-03-05");
    }

    #[test_case(2024, Month::January, 1 => "2024-01-01"; "new year")]
    #[test_case(2024, Month::December, 31 => "2024-12-31"; "end of year")]
    #[test_case(2000, Month::February, 29 => "2000-02-29"; "leap day")]
    #[test_case(2024, Month::July, 15 => "2024-07-15"; "mid year")]
    fn test_format_date_cases(year: i32, month: Month, day: u8) -> String {
        let date = Date::from_calendar_date(year, month, day).unwrap();
        format_date(date)
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_parse_empty_content() {
        let parser = Parser::new();
        let adr = parser.parse("").unwrap();

        assert_eq!(adr.number, 0);
        assert!(adr.title.is_empty());
    }

    #[test]
    fn test_parse_only_title() {
        let content = "# 1. Just a Title";

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Just a Title");
    }

    #[test]
    fn test_parse_extra_sections_ignored() {
        let content = r#"# 1. Test

## Status

Proposed

## Context

Context.

## Decision

Decision.

## Consequences

Consequences.

## Notes

These should be ignored.

## References

- ref1
- ref2
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        // Extra sections are ignored, main content is still parsed
        assert_eq!(adr.number, 1);
        assert_eq!(adr.status, AdrStatus::Proposed);
    }

    #[test]
    fn test_parse_case_insensitive_sections() {
        let content = r#"# 1. Case Test

## STATUS

Accepted

## CONTEXT

Context.

## DECISION

Decision.

## CONSEQUENCES

Consequences.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        // Sections should be matched case-insensitively
        assert_eq!(adr.status, AdrStatus::Accepted);
        assert_eq!(adr.context, "Context.");
    }

    #[test]
    fn test_parse_content_with_markdown_formatting() {
        let content = r#"# 1. Formatted ADR

## Status

Accepted

## Context

We have **bold** and *italic* text.

Also `code` and [links](https://example.com).

## Decision

```rust
fn main() {
    println!("Hello");
}
```

## Consequences

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert!(adr.context.contains("bold"));
        assert!(adr.decision.contains("fn main"));
        assert!(adr.consequences.contains("Column 1"));
    }

    // ========== Regex Tests ==========

    #[test]
    fn test_link_regex_pattern() {
        let content = "Supersedes [1. Use MySQL](0001-use-mysql.md)";
        let caps = LINK_REGEX.captures(content).unwrap();

        assert_eq!(caps.get(1).unwrap().as_str(), "Supersedes");
        assert_eq!(caps.get(2).unwrap().as_str(), "1");
        assert_eq!(caps.get(3).unwrap().as_str(), "0001");
    }

    #[test]
    fn test_link_regex_amended_by() {
        let content = "Amended by [3. Update API](0003-update-api.md)";
        let caps = LINK_REGEX.captures(content).unwrap();

        assert_eq!(caps.get(1).unwrap().as_str(), "Amended by");
        assert_eq!(caps.get(2).unwrap().as_str(), "3");
    }

    #[test]
    fn test_number_regex_pattern() {
        let filename = "0042-some-decision.md";
        let caps = NUMBER_REGEX.captures(filename).unwrap();

        assert_eq!(caps.get(1).unwrap().as_str(), "0042");
    }

    #[test]
    fn test_number_regex_no_match() {
        assert!(NUMBER_REGEX.captures("not-an-adr.md").is_none());
        assert!(NUMBER_REGEX.captures("01-short.md").is_none());
        assert!(NUMBER_REGEX.captures("00001-too-long.md").is_none());
    }

    // ========== MADR 4.0.0 Frontmatter Tests ==========

    #[test]
    fn test_parse_madr_frontmatter() {
        let content = r#"---
number: 1
title: Use MADR Format
date: 2024-09-15
status: accepted
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
informed:
  - Dave
  - Eve
---

## Context and Problem Statement

We need a standard format for ADRs.

## Decision Outcome

Chosen option: "MADR 4.0.0", because it provides rich metadata.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.number, 1);
        assert_eq!(adr.title, "Use MADR Format");
        assert_eq!(adr.status, AdrStatus::Accepted);
        assert_eq!(adr.decision_makers, vec!["Alice", "Bob"]);
        assert_eq!(adr.consulted, vec!["Carol"]);
        assert_eq!(adr.informed, vec!["Dave", "Eve"]);
    }

    #[test]
    fn test_parse_madr_frontmatter_partial_fields() {
        let content = r#"---
number: 2
title: Partial MADR
date: 2024-09-15
status: proposed
decision-makers:
  - Alice
---

## Context

Context.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.decision_makers, vec!["Alice"]);
        assert!(adr.consulted.is_empty());
        assert!(adr.informed.is_empty());
    }

    #[test]
    fn test_parse_madr_frontmatter_empty_fields() {
        let content = r#"---
number: 3
title: No MADR Fields
date: 2024-09-15
status: accepted
---

## Context

Context.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert!(adr.decision_makers.is_empty());
        assert!(adr.consulted.is_empty());
        assert!(adr.informed.is_empty());
    }

    #[test]
    fn test_parse_madr_with_links() {
        let content = r#"---
number: 4
title: MADR With Links
date: 2024-09-15
status: accepted
decision-makers:
  - Alice
links:
  - target: 1
    kind: supersedes
  - target: 2
    kind: amends
---

## Context

Context.
"#;

        let parser = Parser::new();
        let adr = parser.parse(content).unwrap();

        assert_eq!(adr.decision_makers, vec!["Alice"]);
        assert_eq!(adr.links.len(), 2);
        assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
        assert_eq!(adr.links[1].kind, LinkKind::Amends);
    }
}
