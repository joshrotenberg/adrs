# User Issue Validation: zartc (March 2026)

Three issues filed by user **zartc** on 2026-03-03. All three are valid.

---

## Issue #198: ADR014 false positive for ADR #0001

**Status:** Valid bug
**Component:** `mdbook-lint-rulesets` v0.14.2 (upstream dependency)
**File:** `mdbook-lint-rulesets/src/adr/adr014.rs`

### Problem

After `adrs init`, `adrs doctor` warns:

```
warning: [ADR014] Section '## Decision' appears to be empty or contains only
placeholder text [doc/adr/0001-record-architecture-decisions.md:13]
```

The initial ADR #0001 Decision section contains legitimate text:

> "We will use Architecture Decision Records, as **described** by Michael Nygard
> in his article "Documenting Architecture Decisions"."

### Root Cause

`Adr014::is_placeholder_content()` (line 65-81) does naive substring matching:

```rust
// adr014.rs:74-77
for pattern in PLACEHOLDER_PATTERNS.iter() {
    if content_lower.contains(pattern) {
        return true;
    }
}
```

The `PLACEHOLDER_PATTERNS` list (line 13-30) includes `"describe"`. The word
"**describe**d" in the legitimate Decision text matches this substring.

The existing lint test in `crates/adrs-core/src/lint.rs:382` uses different
Decision text (`"We will use Architecture Decision Records."`) that doesn't
contain "describe", so it doesn't catch this.

### Fix Plan

Fix is in `mdbook-lint-rulesets` crate, `src/adr/adr014.rs`:

1. Change `is_placeholder_content()` to use word-boundary matching instead of
   substring contains. Replace the loop with regex-based whole-word matching:

   ```rust
   use regex::Regex;
   use std::sync::LazyLock;

   static PLACEHOLDER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
       Regex::new(r"(?i)\b(?:todo|tbd|to be determined|to be decided|fill in|placeholder|describe|add content|write here|xxx|lorem ipsum)\b").unwrap()
   });

   // And for non-word patterns:
   static PLACEHOLDER_LITERALS: &[&str] = &["...", "[insert", "<insert"];
   ```

   Then update the function:
   ```rust
   fn is_placeholder_content(content: &str) -> bool {
       let trimmed = content.trim();
       if trimmed.is_empty() || trimmed.len() < 3 {
           return true;
       }
       if PLACEHOLDER_REGEX.is_match(trimmed) {
           return true;
       }
       let lower = trimmed.to_lowercase();
       PLACEHOLDER_LITERALS.iter().any(|p| lower.contains(p))
   }
   ```

2. Add a test case using the actual ADR #0001 Decision text:

   ```rust
   #[test]
   fn test_no_false_positive_on_initial_adr() {
       let content = r#"# 1. Record architecture decisions

   Date: 2024-01-15

   ## Status

   Accepted

   ## Context

   We need to record the architectural decisions made on this project.

   ## Decision

   We will use Architecture Decision Records, as described by Michael Nygard in his article "Documenting Architecture Decisions".

   ## Consequences

   See Michael Nygard's article, linked above. For a lightweight ADR toolset, see Nat Pryce's adr-tools.
   "#;
       let doc = create_test_document(content);
       let rule = Adr014::default();
       let violations = rule.check(&doc).unwrap();
       assert!(violations.is_empty(), "False positive on initial ADR #0001");
   }
   ```

3. Also update the test in `crates/adrs-core/src/lint.rs:366-413`
   (`test_lint_valid_nygard_adr`) to use the actual ADR #0001 text (with
   "as described by") so it catches regressions.

---

## Issue #197: ADR010 false positive for superseded ADRs

**Status:** Valid bug
**Component:** `mdbook-lint-rulesets` v0.14.2 (upstream dependency)
**File:** `mdbook-lint-rulesets/src/adr/adr010.rs`

### Problem

After superseding an ADR:

```
adrs new "use PostgreSQL"
adrs new -s 2 "Use MySQL"
adrs doctor
```

Doctor warns:

```
warning: [ADR010] 0002-use-postgresql.md: Superseded ADR should reference the
ADR that replaces it
```

But the file clearly contains the reference:

```markdown
## Status

Superseded

Superseded by [3. Use MySQL](0003-use-mysql.md)
```

### Root Cause

`Adr010::has_adr_reference()` (line 44-46) uses two regexes, **neither of which
matches the link format that `adrs` generates**:

1. `ADR_REFERENCE_REGEX` (line 15-16): `(?i)ADR[-\s]?(\d+)`
   - Matches "ADR-001" or "ADR 1" text patterns.
   - The generated text `Superseded by [3. Use MySQL](0003-use-mysql.md)`
     contains no "ADR" literal, so this doesn't match.

2. `ADR_LINK_REGEX` (line 19-21): `\[.*?\]\([^)]*?(?:adr|ADR)[/\\]?\d+[^)]*\.md\)`
   - Requires `adr/` or `ADR/` directory prefix in the link URL.
   - `adrs` generates **relative bare filenames** like `0003-use-mysql.md`
     (no directory prefix), so this doesn't match.

The link format `adrs` generates comes from `Adr::filename()` in
`crates/adrs-core/src/types.rs:88-90`, which produces `NNNN-slug.md`. The
supersede link is written by `update_legacy_metadata()` in
`crates/adrs-core/src/repository.rs:493`. Both `adrs` and the original
`adr-tools` use this bare filename format.

### Fix Plan

Fix is in `mdbook-lint-rulesets` crate, `src/adr/adr010.rs`:

1. Broaden `ADR_LINK_REGEX` to also match bare `NNNN-*.md` filenames (the
   standard ADR naming convention):

   ```rust
   static ADR_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
       Regex::new(r"\[.*?\]\([^)]*?\d{4}-[^)]*\.md\)").expect("Invalid regex")
   });
   ```

   Or keep the existing regex and add a second pattern:

   ```rust
   static ADR_BARE_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
       Regex::new(r"\[.*?\]\(\d{4}-[^)]*\.md\)").expect("Invalid regex")
   });
   ```

   Then update `has_adr_reference`:
   ```rust
   fn has_adr_reference(content: &str) -> bool {
       ADR_REFERENCE_REGEX.is_match(content)
           || ADR_LINK_REGEX.is_match(content)
           || ADR_BARE_LINK_REGEX.is_match(content)
   }
   ```

2. Add a test case matching the exact format `adrs` produces:

   ```rust
   #[test]
   fn test_superseded_with_bare_filename_link() {
       let docs = vec![create_nygard_adr(
           "Superseded",
           "\nSuperseded by [3. Use MySQL](0003-use-mysql.md)",
       )];
       let rule = Adr010;
       let violations = rule.check_collection(&docs).unwrap();
       assert!(violations.is_empty(), "Should recognize bare filename ADR links");
   }
   ```

---

## Issue #196: `adrs new` opens a temp file for editing

**Status:** Valid UX issue
**Component:** `adrs` CLI crate
**Files:** `crates/adrs/src/commands/new.rs:117-121`,
`crates/adrs/src/commands/edit.rs:12-15`

### Problem

- `adrs new "title"` opens a temporary file in `$EDITOR` instead of the actual
  ADR file
- `adrs edit xxxx` opens a temporary copy instead of the real file

The user expects the editor to open the actual ADR file at its real path, like
`adr-tools` does.

### Root Cause

Both commands use the `edit` crate (v0.1) via `edit::edit(&content)`. This
function by design:

1. Creates a temp file
2. Writes the content string to it
3. Opens `$EDITOR` on the temp file
4. Reads back the modified content
5. Returns the string

The user sees a temp file path in their editor title bar. Changes are written
back to the real file after the editor closes, but the UX is confusing.

The current code in `edit.rs`:

```rust
let content = repo.read_content(&adr)?;
let edited = edit::edit(&content).context("Failed to open editor")?;
let path = repo.write_content(&adr, &edited)?;
```

And in `new.rs:117-121`:

```rust
if !no_edit {
    let content = repo.read_content(&adr)?;
    let edited = edit::edit(&content).context("Failed to open editor")?;
    repo.write_content(&adr, &edited)?;
}
```

### Fix Plan

The `edit` crate (v0.1) already provides `edit::edit_file(path)` which opens
`$EDITOR` directly on a given file path (no temp file). The current code uses
`edit::edit(&content)` (the temp-file variant) when it should use
`edit::edit_file()`.

1. Update `crates/adrs/src/commands/edit.rs`:

   ```rust
   pub fn edit(root: &Path, query: &str) -> Result<()> {
       let repo = Repository::open(root)
           .context("ADR repository not found. Run 'adrs init' first.")?;
       let adr = repo.find(query).context("ADR not found")?;
       let path = adr.path.clone()
           .unwrap_or_else(|| repo.adr_path().join(adr.filename()));

       edit::edit_file(&path).context("Failed to open editor")?;

       println!("{}", path.display());
       Ok(())
   }
   ```

2. Update `crates/adrs/src/commands/new.rs` (lines 117-121):

   ```rust
   if !no_edit {
       edit::edit_file(&path).context("Failed to open editor")?;
   }
   ```

3. Verify no other code uses `edit::edit` — only `new.rs` and `edit.rs` do.
   The `edit` crate dependency stays (we still use it, just a different function).
