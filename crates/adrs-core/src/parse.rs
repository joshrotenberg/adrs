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
                    adr.links.push(AdrLink::new(target, kind));
                }
            } else if !line.contains('[') && !line.contains(']') {
                // Plain status text (not a link line)
                // Only set status if it looks like a simple status word
                let word = line.split_whitespace().next().unwrap_or("");
                if matches!(
                    word.to_lowercase().as_str(),
                    "proposed" | "accepted" | "deprecated" | "superseded" | "draft" | "rejected"
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

    #[test]
    fn test_today() {
        let date = today();
        assert!(date.year() >= 2024);
    }

    #[test]
    fn test_format_date() {
        let date = Date::from_calendar_date(2024, Month::March, 5).unwrap();
        assert_eq!(format_date(date), "2024-03-05");
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
}
