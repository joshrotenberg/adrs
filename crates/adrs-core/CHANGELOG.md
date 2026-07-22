# Changelog

All notable changes to this project will be documented in this file.

## [0.10.1] - 2026-07-22

### Bug Fixes

- Preserve line endings and blank-line separators in body patches (closes #339, closes #340)
- Map MADR H3 Consequences under Decision Outcome to consequences on read (closes #338)
- Preserve CRLF line endings in legacy-mode metadata writes (closes #344)
- Separate consecutive list items in parse_sections (closes #346)
- Detect ADR format from headings so ng-init repos pass doctor (closes #348)

### Features

- Add MCP init_repository tool and start server at unconfigured paths (closes #349)


## [0.10.0] - 2026-07-16

### Bug Fixes

- Preserve MADR 4.0.0 content when updating ADR body sections
- Patch only changed body sections on MADR 4.0.0 update
- Route Nygard consequences patches to ## Consequences H2
- Preserve markdown on body-only Repository::update
- Tighten consequences routing and reject empty update_content
- Rewrite frontmatter metadata via YAML Mapping
- Remove brittle people-field YAML equality guard
- Track fenced code with CommonMark char and run length
- Separate appended MADR Consequences after suppressed newline
- Detect Consequences H2 anywhere outside fences
- Describe adr-tools as shell implementation in init ADR #0001
- Normalize rendered templates to end with exactly one newline (closes #320)

### Documentation

- Clarify BodySectionPatch migration and refresh ADR 0009

### Features

- Update init ADR #0001 seed with markdown links and trailing newline

### Miscellaneous

- Migrate homebrew distribution to homebrew-core

### Testing

- Expand BodySectionPatch preservation coverage for issue #310
- Pin write-path classes for people YAML, fences, and newlines
- Absorb corpus people-YAML fixtures for status and link write paths
- Use descriptive comments for write-path pins
- Pin MADR Consequences H2 before Decision Outcome


## [0.9.0] - 2026-07-10

### Bug Fixes

- Reject empty or whitespace-only status values (closes #305)
- Parse CRLF frontmatter and legacy Date lines (closes #326, #324)
- Preserve link descriptions and hrefs in metadata writer
- Add Date line to nygard bare-minimal template (closes #330)

### Documentation

- Fix drift found by v0.9.0 release readiness audit

### Features

- Configurable doctor rules and warnings-as-errors via adrs.toml

### Testing

- Wire up adr-corpus fixtures as a round-trip harness (closes #318)
- Pin CRLF frontmatter behavior and force LF checkout for corpus fixtures


## [0.8.0] - 2026-06-15

### Features

- Allow configuring default status
- Make --no-edit default configurable in adrs.toml (closes #298)
- Configurable default TOC prefix via [generate].toc_prefix in adrs.toml (closes #299)
- Add export.base_url config for adrs export json (closes #300)


## [0.7.6] - 2026-06-08


## [0.7.5] - 2026-06-08

### Bug Fixes

- MADR bare template no longer emits unparseable null YAML

### Documentation

- Expand adrs-core crate-level documentation

### Features

- Polish adrs-core for external library use

### Miscellaneous

- Complete crate metadata (docs, homepage, keywords, categories)
- Declare per-crate MSRV with a CI guard


## [0.7.4] - 2026-06-06

### Bug Fixes

- --ng flag now overrides template mode for existing repos
- Accept string or list for frontmatter fields, report parse errors in doctor
- Resolve clippy 1.96 lints (sort_by_key, collapsible_match)
- Serialize env-var tests to remove ADR_DIRECTORY race

### Testing

- Add tests for string-or-vec parsing and doctor parse error reporting
- Add missing tests for template, export, cli, search, lint, config (closes #235, closes #236, closes #237, closes #238, closes #239, closes #241)


## [0.7.3] - 2026-03-04

### Bug Fixes

- Open actual ADR file in editor instead of temp file
- Bump mdbook-lint-rulesets to 0.14.3


## [0.7.2] - 2026-02-26

### Bug Fixes

- Trim extra newline before frontmatter closing separator ([#192](https://github.com/joshrotenberg/adrs/pull/192))


## [0.7.1] - 2026-02-24

### Bug Fixes

- Preserve file content when updating ADR metadata ([#187](https://github.com/joshrotenberg/adrs/pull/187))
- Fall back to body H1 when frontmatter title missing, implement pad filter

### Testing

- Add real-world ADR corpus integration tests


## [0.7.0] - 2026-02-20

### Bug Fixes

- Generate functional supersedes/superseded-by markdown links

### Styling

- Run cargo fmt


## [0.6.2] - 2026-02-11

### Bug Fixes

- Honor adrs.toml template fields and accept "nextgen" mode
- Wire custom template config, validate empty adr_dir, add tests
- Resolve security audit failures and update dependencies


## [0.6.1] - 2026-01-27

### Bug Fixes

- MADR format ADRs not parsed correctly
- Persist tags in ADR YAML frontmatter


## [0.6.0] - 2026-01-26

### Bug Fixes

- Init detects existing ADRs and skips initial ADR creation

### Features

- Add source_uri field to JSON-ADR spec for federation
- Add template management commands
- Add tags support for ADR categorization
- Simplify link command with auto-derived reverse links


## [0.5.1] - 2026-01-22

### Features

- Add status command to change ADR status


## [0.5.0] - 2026-01-22

### Bug Fixes

- Align MADR templates with official adr/madr repository

### Features

- Add doctor command for repository health checks
- Add config discovery with directory tree search

