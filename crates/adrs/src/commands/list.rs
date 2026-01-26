//! List ADRs command.

use adrs_core::{Adr, AdrStatus, Repository};
use anyhow::{Context, Result};
use std::path::Path;
use time::Date;

/// List ADRs with optional filtering.
pub fn list(
    root: &Path,
    status_filter: Option<String>,
    since: Option<String>,
    until: Option<String>,
    decider: Option<String>,
    tag: Option<String>,
    long_format: bool,
) -> Result<()> {
    let repo =
        Repository::open(root).context("ADR repository not found. Run 'adrs init' first.")?;

    let adrs = repo.list()?;

    // Parse date filters
    let since_date = parse_date_filter(&since)?;
    let until_date = parse_date_filter(&until)?;

    // Parse status filter
    let status_filter: Option<AdrStatus> = status_filter.map(|s| s.parse().unwrap());

    // Filter ADRs
    let filtered: Vec<&Adr> = adrs
        .iter()
        .filter(|adr| {
            matches_filters(
                adr,
                &status_filter,
                &since_date,
                &until_date,
                &decider,
                &tag,
            )
        })
        .collect();

    // Output
    for adr in filtered {
        if long_format {
            print_long_format(adr);
        } else {
            print_short_format(adr);
        }
    }

    Ok(())
}

/// Parse a date string in YYYY-MM-DD format.
fn parse_date_filter(date_str: &Option<String>) -> Result<Option<Date>> {
    match date_str {
        Some(s) => {
            let date = Date::parse(s, &time::format_description::well_known::Iso8601::DATE)
                .with_context(|| format!("Invalid date format: '{}'. Use YYYY-MM-DD.", s))?;
            Ok(Some(date))
        }
        None => Ok(None),
    }
}

/// Check if an ADR matches all the provided filters.
fn matches_filters(
    adr: &Adr,
    status_filter: &Option<AdrStatus>,
    since_date: &Option<Date>,
    until_date: &Option<Date>,
    decider: &Option<String>,
    tag: &Option<String>,
) -> bool {
    // Status filter (case-insensitive match)
    if let Some(filter_status) = status_filter
        && !status_matches(&adr.status, filter_status)
    {
        return false;
    }

    // Since date filter
    if let Some(since) = since_date
        && adr.date < *since
    {
        return false;
    }

    // Until date filter
    if let Some(until) = until_date
        && adr.date > *until
    {
        return false;
    }

    // Decider filter (case-insensitive substring match)
    if let Some(decider_name) = decider {
        let decider_lower = decider_name.to_lowercase();
        let has_decider = adr
            .decision_makers
            .iter()
            .any(|dm| dm.to_lowercase().contains(&decider_lower));
        if !has_decider {
            return false;
        }
    }

    // Tag filter (case-insensitive match)
    if let Some(tag_filter) = tag {
        let tag_lower = tag_filter.to_lowercase();
        let has_tag = adr.tags.iter().any(|t| t.to_lowercase() == tag_lower);
        if !has_tag {
            return false;
        }
    }

    true
}

/// Check if two statuses match (case-insensitive).
fn status_matches(adr_status: &AdrStatus, filter_status: &AdrStatus) -> bool {
    match (adr_status, filter_status) {
        (AdrStatus::Proposed, AdrStatus::Proposed) => true,
        (AdrStatus::Accepted, AdrStatus::Accepted) => true,
        (AdrStatus::Deprecated, AdrStatus::Deprecated) => true,
        (AdrStatus::Superseded, AdrStatus::Superseded) => true,
        (AdrStatus::Custom(a), AdrStatus::Custom(b)) => a.to_lowercase() == b.to_lowercase(),
        // Allow matching custom status against standard ones by name
        (AdrStatus::Custom(s), standard) | (standard, AdrStatus::Custom(s)) => {
            s.to_lowercase() == standard.to_string().to_lowercase()
        }
        _ => false,
    }
}

/// Print ADR in short format (just the path).
fn print_short_format(adr: &Adr) {
    if let Some(path) = &adr.path {
        println!("{}", path.display());
    } else {
        println!("{}", adr.filename());
    }
}

/// Print ADR in long format (number, status, date, title).
fn print_long_format(adr: &Adr) {
    let date = adr
        .date
        .format(&time::format_description::well_known::Iso8601::DATE)
        .unwrap_or_default();

    println!(
        "{:4}  {:12}  {}  {}",
        adr.number, adr.status, date, adr.title
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn create_test_adr(number: u32, status: AdrStatus, date: Date) -> Adr {
        let mut adr = Adr::new(number, format!("Test ADR {}", number));
        adr.status = status;
        adr.date = date;
        adr
    }

    #[test]
    fn test_status_matches_same() {
        assert!(status_matches(&AdrStatus::Accepted, &AdrStatus::Accepted));
        assert!(status_matches(&AdrStatus::Proposed, &AdrStatus::Proposed));
        assert!(status_matches(
            &AdrStatus::Deprecated,
            &AdrStatus::Deprecated
        ));
        assert!(status_matches(
            &AdrStatus::Superseded,
            &AdrStatus::Superseded
        ));
    }

    #[test]
    fn test_status_matches_different() {
        assert!(!status_matches(&AdrStatus::Accepted, &AdrStatus::Proposed));
        assert!(!status_matches(
            &AdrStatus::Proposed,
            &AdrStatus::Deprecated
        ));
    }

    #[test]
    fn test_status_matches_custom() {
        assert!(status_matches(
            &AdrStatus::Custom("Draft".to_string()),
            &AdrStatus::Custom("draft".to_string())
        ));
        assert!(status_matches(
            &AdrStatus::Custom("DRAFT".to_string()),
            &AdrStatus::Custom("draft".to_string())
        ));
    }

    #[test]
    fn test_matches_filters_status() {
        let date = Date::from_calendar_date(2024, Month::January, 15).unwrap();
        let adr = create_test_adr(1, AdrStatus::Accepted, date);

        assert!(matches_filters(
            &adr,
            &Some(AdrStatus::Accepted),
            &None,
            &None,
            &None,
            &None
        ));
        assert!(!matches_filters(
            &adr,
            &Some(AdrStatus::Proposed),
            &None,
            &None,
            &None,
            &None
        ));
    }

    #[test]
    fn test_matches_filters_tag() {
        let date = Date::from_calendar_date(2024, Month::January, 15).unwrap();
        let mut adr = create_test_adr(1, AdrStatus::Accepted, date);
        adr.tags = vec!["security".to_string(), "api".to_string()];

        // Tag matches (case-insensitive)
        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &None,
            &Some("security".to_string())
        ));
        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &None,
            &Some("SECURITY".to_string())
        ));
        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &None,
            &Some("api".to_string())
        ));

        // Tag doesn't match
        assert!(!matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &None,
            &Some("database".to_string())
        ));
    }

    #[test]
    fn test_matches_filters_since() {
        let date = Date::from_calendar_date(2024, Month::June, 15).unwrap();
        let adr = create_test_adr(1, AdrStatus::Accepted, date);

        let before = Date::from_calendar_date(2024, Month::January, 1).unwrap();
        let after = Date::from_calendar_date(2024, Month::December, 1).unwrap();

        assert!(matches_filters(
            &adr,
            &None,
            &Some(before),
            &None,
            &None,
            &None
        ));
        assert!(!matches_filters(
            &adr,
            &None,
            &Some(after),
            &None,
            &None,
            &None
        ));
    }

    #[test]
    fn test_matches_filters_until() {
        let date = Date::from_calendar_date(2024, Month::June, 15).unwrap();
        let adr = create_test_adr(1, AdrStatus::Accepted, date);

        let before = Date::from_calendar_date(2024, Month::January, 1).unwrap();
        let after = Date::from_calendar_date(2024, Month::December, 1).unwrap();

        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &Some(after),
            &None,
            &None
        ));
        assert!(!matches_filters(
            &adr,
            &None,
            &None,
            &Some(before),
            &None,
            &None
        ));
    }

    #[test]
    fn test_matches_filters_decider() {
        let date = Date::from_calendar_date(2024, Month::January, 15).unwrap();
        let mut adr = create_test_adr(1, AdrStatus::Accepted, date);
        adr.decision_makers = vec!["Alice Smith".to_string(), "Bob Jones".to_string()];

        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &Some("alice".to_string()),
            &None
        ));
        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &Some("Smith".to_string()),
            &None
        ));
        assert!(matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &Some("bob".to_string()),
            &None
        ));
        assert!(!matches_filters(
            &adr,
            &None,
            &None,
            &None,
            &Some("charlie".to_string()),
            &None
        ));
    }

    #[test]
    fn test_matches_filters_combined() {
        let date = Date::from_calendar_date(2024, Month::June, 15).unwrap();
        let mut adr = create_test_adr(1, AdrStatus::Accepted, date);
        adr.decision_makers = vec!["Alice".to_string()];
        adr.tags = vec!["security".to_string()];

        let since = Date::from_calendar_date(2024, Month::January, 1).unwrap();
        let until = Date::from_calendar_date(2024, Month::December, 1).unwrap();

        // All filters match
        assert!(matches_filters(
            &adr,
            &Some(AdrStatus::Accepted),
            &Some(since),
            &Some(until),
            &Some("Alice".to_string()),
            &Some("security".to_string())
        ));

        // Tag doesn't match
        assert!(!matches_filters(
            &adr,
            &Some(AdrStatus::Accepted),
            &Some(since),
            &Some(until),
            &Some("Alice".to_string()),
            &Some("database".to_string())
        ));
    }

    #[test]
    fn test_parse_date_filter_valid() {
        let result = parse_date_filter(&Some("2024-01-15".to_string())).unwrap();
        assert!(result.is_some());
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), Month::January);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_date_filter_none() {
        let result = parse_date_filter(&None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_date_filter_invalid() {
        let result = parse_date_filter(&Some("not-a-date".to_string()));
        assert!(result.is_err());
    }
}
