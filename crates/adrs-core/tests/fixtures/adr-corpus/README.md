# ADR Test Corpus

This directory contains synthetic ADRs covering formats and edge cases found
in the wild. The corpus is exercised by `crates/adrs-core/tests/adr_corpus.rs`,
which runs every file through the parser and the repository read/write paths
(see issue #318).

Some fixtures intentionally exercise behavior that is currently lossy; the
harness pins those cases with KNOWN-LOSSY / KNOWN-QUIRK comments and tracking
issues. When adding a fixture, keep the `NNNN-` numbering contiguous and add a
row here.

## Files

| File | Format | Status | Features |
|------|--------|--------|----------|
| 0001-record-architecture-decisions.md | Nygard | Accepted | Basic ADR |
| 0002-use-postgresql-for-persistence.md | Nygard | Accepted | Supersedes link |
| 0003-use-rust-for-backend.md | Nygard | Proposed | Multi-line consequences; filename deliberately differs from slugified title |
| 0004-use-madr-format.md | MADR 4.0 | Accepted | YAML frontmatter, decision-makers, consulted, informed |
| 0005-api-versioning-strategy.md | Nygard | Accepted | Amends link to a hand-named file |
| 0006-deprecated-xml-api.md | Nygard | Deprecated (parses as Superseded) | Superseded by link overrides declared status |
| 0007-authentication-mechanism.md | MADR 4.0 | Accepted | Structured frontmatter links with descriptions |
| 0008-fenced-heading-examples.md | MADR 4.0 frontmatter | Accepted | Fenced code block containing heading-lookalike lines |
| 0009-no-trailing-newline.md | Nygard | Accepted | File ends without a trailing newline |
| 0010-non-canonical-status.md | Nygard | Free-form prose (parses as Proposed, #310) | Non-canonical status text |
| 0011-people-yaml-block-scalar.md | MADR 4.0 frontmatter | Proposed | `consulted: >-` folded block scalar |
| 0012-people-yaml-zero-indent.md | MADR 4.0 frontmatter | Proposed | Zero-indent `consulted` list items |
| 0013-people-yaml-comment-between.md | MADR 4.0 frontmatter | Proposed | Comment between `consulted` key and items |
| 0014-people-yaml-four-space.md | MADR 4.0 frontmatter | Proposed | Four-space-indented `consulted` list |
| 0015-nested-fence-backticks.md | MADR 4.0 frontmatter | Accepted | Outer `` ```` `` wrapping inner `` ``` `` + heading lookalike |
| 0016-tilde-fence-wrapping-backticks.md | MADR 4.0 frontmatter | Accepted | Outer `~~~` fence wrapping backtick sample (Markdown, not YAML) |

## Sources

These ADRs are synthetic but inspired by real-world examples from:
- adr-tools (Nygard format)
- MADR 4.0.0 template
- Backstage ADRs
- Microsoft Engineering Playbook
