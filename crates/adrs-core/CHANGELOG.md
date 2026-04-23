# Changelog

All notable changes to this project will be documented in this file.

## [0.7.4] - 2026-04-23

### Bug Fixes

- --ng flag now overrides template mode for existing repos
- Accept string or list for frontmatter fields, report parse errors in doctor

### Testing

- Add tests for string-or-vec parsing and doctor parse error reporting


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

