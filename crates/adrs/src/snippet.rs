//! Snippet extraction for search results.
//!
//! Shared by the `search` command and the MCP `search_adrs` tool so both
//! produce identical output and stay UTF-8 safe.

/// Bytes of context to include on either side of a match.
const CONTEXT_BYTES: usize = 40;

/// Characters shown as a preview when there is no match to center on.
const PREVIEW_CHARS: usize = 80;

/// Round `index` down to the nearest UTF-8 character boundary in `text`.
///
/// `str::floor_char_boundary` is still unstable, and the offsets we derive from
/// byte arithmetic can land inside a multi-byte character.
fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Lowercase `text`, returning it alongside a map from each byte offset in the
/// lowercased string back to the offset of the character that produced it.
///
/// Lowercasing can change byte length (`İ` U+0130 lowercases to two
/// characters), so offsets found in the lowercased string are not valid
/// indices into the original text without this translation. The map has one
/// extra entry so that the end of the lowercased string maps to `text.len()`.
fn lowercase_with_offsets(text: &str) -> (String, Vec<usize>) {
    // Lowercase with `str::to_lowercase` rather than character by character.
    // Both callers gate on `text.to_lowercase().contains(query)`, and only the
    // string form applies context-sensitive rules such as word-final sigma
    // (`\u{3a3}` becomes `\u{3c2}`, not `\u{3c3}`). Lowering char by char here
    // would make a section report a match that this function cannot locate.
    let lowered = text.to_lowercase();

    let mut offsets = Vec::with_capacity(lowered.len() + 1);
    for (offset, ch) in text.char_indices() {
        let end = offsets.len() + ch.to_lowercase().map(char::len_utf8).sum::<usize>();
        offsets.resize(end.min(lowered.len()), offset);
    }
    // The two lowercasings agree on byte length for every Unicode scalar
    // today. Reconcile anyway so indexing stays in bounds if that ever changes.
    offsets.resize(lowered.len(), text.len());
    offsets.push(text.len());

    (lowered, offsets)
}

/// Locate `query` in `text`, returning its byte range in the *original* text.
///
/// `query` must already be lowercased when `case_sensitive` is false. Both
/// returned offsets are character boundaries in `text`.
fn find_match_range(text: &str, query: &str, case_sensitive: bool) -> Option<(usize, usize)> {
    if case_sensitive {
        return text.find(query).map(|pos| (pos, pos + query.len()));
    }

    let (lowered, offsets) = lowercase_with_offsets(text);
    lowered
        .find(query)
        .map(|pos| (offsets[pos], offsets[pos + query.len()]))
}

/// Extract a snippet of `text` centered on the first match of `query`.
///
/// `query` must already be lowercased when `case_sensitive` is false. Falls
/// back to a preview of the start of `text` when there is no match.
pub(crate) fn extract_snippet(text: &str, query: &str, case_sensitive: bool) -> String {
    let Some((match_start, match_end)) = find_match_range(text, query, case_sensitive) else {
        let preview: String = text.chars().take(PREVIEW_CHARS).collect();
        let truncated = preview.len() < text.len();
        let preview = preview.replace('\n', " ");
        return if truncated {
            format!("{}...", preview)
        } else {
            preview
        };
    };

    let start = floor_char_boundary(text, match_start.saturating_sub(CONTEXT_BYTES));
    let end = floor_char_boundary(text, match_end.saturating_add(CONTEXT_BYTES));

    // Expand outward so the snippet starts and ends on whole words.
    let start = match text[..start].rfind(char::is_whitespace) {
        Some(pos) => pos + text[pos..].chars().next().map_or(1, char::len_utf8),
        None => start,
    };
    let end = text[end..]
        .find(char::is_whitespace)
        .map_or(end, |pos| end + pos);

    let mut snippet = text[start..end].to_string();

    if start > 0 {
        snippet = format!("...{}", snippet);
    }
    if end < text.len() {
        snippet = format!("{}...", snippet);
    }

    // Replace newlines with spaces for cleaner single-line output.
    snippet.replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_in_middle() {
        let text = "This is some context about the database decision and consequences.";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("database"));
    }

    #[test]
    fn match_at_start() {
        let text = "database is the topic of this context section.";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("database"));
    }

    #[test]
    fn match_at_end() {
        let text = "This context is about the database";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("database"));
    }

    #[test]
    fn no_match_returns_whole_short_text() {
        let text = "This context has no matching term at all.";
        assert_eq!(extract_snippet(text, "nonexistent", false), text);
    }

    #[test]
    fn no_match_truncates_long_text() {
        let text = "a".repeat(100);
        let snippet = extract_snippet(&text, "nonexistent", false);
        assert!(snippet.ends_with("..."));
        assert!(snippet.len() <= PREVIEW_CHARS + 3);
    }

    #[test]
    fn case_insensitive_match_keeps_original_casing() {
        let text = "The DATABASE decision was made.";
        let snippet = extract_snippet(text, "database", false);
        assert!(snippet.contains("DATABASE"));
    }

    #[test]
    fn adds_ellipsis_for_truncated_start() {
        let text = format!("{}match{}", "x ".repeat(25), " y".repeat(25));
        let snippet = extract_snippet(&text, "match", false);
        assert!(snippet.contains("match"));
        assert!(snippet.starts_with("..."));
    }

    #[test]
    fn empty_text_does_not_panic() {
        assert_eq!(extract_snippet("", "query", false), "");
    }

    // ===== UTF-8 regression coverage (issue #379) =====

    #[test]
    fn multibyte_context_does_not_split_characters() {
        // The +/- 40 byte window lands inside a three-byte character.
        let text = format!("{}needle{}", "あ".repeat(30), "い".repeat(30));
        let snippet = extract_snippet(&text, "needle", true);
        assert!(snippet.contains("needle"));
    }

    #[test]
    fn multibyte_match_at_start_and_end() {
        let leading = format!("needle{}", "あ".repeat(30));
        assert!(extract_snippet(&leading, "needle", true).contains("needle"));

        let trailing = format!("{}needle", "あ".repeat(30));
        assert!(extract_snippet(&trailing, "needle", true).contains("needle"));
    }

    #[test]
    fn lowercasing_that_grows_the_text_stays_in_bounds() {
        // U+0130 is two bytes but lowercases to two characters (three bytes),
        // so match offsets in the lowercased text run past the original.
        let text = format!("{}needle", "İ".repeat(50));
        let snippet = extract_snippet(&text, "needle", false);
        assert!(snippet.contains("needle"));
    }

    #[test]
    fn lowercasing_that_grows_the_text_with_trailing_content() {
        let text = format!("{}needle{}", "İ".repeat(50), "İ".repeat(50));
        let snippet = extract_snippet(&text, "needle", false);
        assert!(snippet.contains("needle"));
    }

    #[test]
    fn multibyte_whitespace_does_not_split_characters() {
        // U+3000 (ideographic space) is three bytes; stepping over it by one
        // byte would land mid-character.
        let text = format!(
            "{}\u{3000}needle\u{3000}{}",
            "あ".repeat(30),
            "い".repeat(30)
        );
        let snippet = extract_snippet(&text, "needle", true);
        assert!(snippet.contains("needle"));
    }

    #[test]
    fn no_match_preview_of_multibyte_text() {
        let text = "あ".repeat(100);
        let snippet = extract_snippet(&text, "nonexistent", false);
        assert!(snippet.ends_with("..."));
        assert_eq!(snippet.chars().count(), PREVIEW_CHARS + 3);
    }

    #[test]
    fn no_match_preview_of_short_multibyte_text_has_no_ellipsis() {
        // Under 80 characters but over 80 bytes: nothing was truncated.
        let text = "あ".repeat(50);
        assert_eq!(extract_snippet(&text, "nonexistent", false), text);
    }

    #[test]
    fn word_final_sigma_agrees_with_the_callers_gate() {
        // `str::to_lowercase` maps a word-final sigma to U+03C2; lowering char
        // by char always yields U+03C3. The callers gate the section on the
        // string form, so extraction must use it too or it reports a match it
        // cannot then locate, and the user gets a snippet with no match in it.
        let text = format!(
            "{} \u{3a3}\u{394}\u{39f}\u{3a3} was chosen",
            "x".repeat(200)
        );
        let query = "\u{3a3}\u{394}\u{39f}\u{3a3}".to_lowercase();

        assert!(
            text.to_lowercase().contains(&query),
            "precondition: the callers' gate reports a match"
        );
        let snippet = extract_snippet(&text, &query, false);
        assert!(
            snippet.contains("\u{3a3}\u{394}\u{39f}\u{3a3}"),
            "snippet lost the match: {snippet}"
        );
    }

    #[test]
    fn no_match_preview_is_single_line() {
        // Results print as `{section}: {snippet}`, so a snippet must not wrap.
        let text = "line one\nline two\nline three";
        let snippet = extract_snippet(text, "nonexistent", false);
        assert!(
            !snippet.contains('\n'),
            "preview must be one line: {snippet:?}"
        );
    }

    #[test]
    fn offsets_map_back_to_the_original_text() {
        let text = "İ needle";
        let (start, end) = find_match_range(text, "needle", false).expect("match");
        assert_eq!(&text[start..end], "needle");
    }

    #[test]
    fn snippets_never_panic_on_arbitrary_multibyte_positions() {
        // Sweep the match across every position in a multi-byte haystack.
        for prefix in 0..60 {
            for suffix in 0..3 {
                let text = format!("{}needle{}", "あ".repeat(prefix), "い".repeat(suffix));
                assert!(extract_snippet(&text, "needle", true).contains("needle"));
            }
        }
    }
}
