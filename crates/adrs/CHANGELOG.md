# Changelog

All notable changes to this project will be documented in this file.

## [0.10.1] - 2026-07-22

### Bug Fixes

- Map MADR H3 Consequences under Decision Outcome to consequences on read (closes #338)
- Reject POSIX-style absolute adr_dir on all platforms in init_repository

### Features

- Add MCP init_repository tool and start server at unconfigured paths (closes #349)
- Add --deciders/--consulted/--informed flags to adrs new (closes #85)


## [0.10.0] - 2026-07-16

### Bug Fixes

- Preserve MADR 4.0.0 content when updating ADR body sections
- Patch only changed body sections on MADR 4.0.0 update
- Route Nygard consequences patches to ## Consequences H2
- Preserve markdown on body-only Repository::update
- Tighten consequences routing and reject empty update_content
- Document update_content requiring a body field

### Features

- Update init ADR #0001 seed with markdown links and trailing newline

### Miscellaneous

- Migrate homebrew distribution to homebrew-core

### Testing

- Expand BodySectionPatch preservation coverage for issue #310
- Pin write-path classes for people YAML, fences, and newlines
- Use descriptive comments for write-path pins


## [0.9.0] - 2026-07-10

### Bug Fixes

- Reject empty or whitespace-only status values (closes #305)
- Surface --ng as a no-op for adrs doctor (closes #306)
- Resolve relative CLI paths against the -C working directory

### Documentation

- Fix drift found by v0.9.0 release readiness audit
- Add bare-minimal to MCP create_adr variant error message

### Features

- Configurable doctor rules and warnings-as-errors via adrs.toml

### Testing

- Assert against original status instead of hardcoded Proposed


## [0.8.0] - 2026-06-15

### Features

- Allow configuring default status
- Make --no-edit default configurable in adrs.toml (closes #298)
- Configurable default TOC prefix via [generate].toc_prefix in adrs.toml (closes #299)
- Add export.base_url config for adrs export json (closes #300)


## [0.7.6] - 2026-06-08

### Features

- Add styled help with ENVIRONMENT VARIABLES and CONFIGURATION sections (closes #290)

### Polish

- Align CLI help layout with jpx/roba + version footer + higher-contrast color


## [0.7.5] - 2026-06-08

### Features

- Add --template flag to adrs new for custom template files (closes #122)

### Miscellaneous

- Complete crate metadata (docs, homepage, keywords, categories)
- Declare per-crate MSRV with a CI guard


## [0.7.4] - 2026-06-06

### Bug Fixes

- --ng flag now overrides template mode for existing repos
- Reject invalid status/link_type in MCP tools, add missing error path tests (closes #229, closes #240)

### Documentation

- Fix high-priority documentation inaccuracies

### Features

- Migrate MCP server from rmcp to tower-mcp
- Upgrade tower-mcp from 0.1 to 0.9
- Add format/variant/MADR fields and tags to MCP create_adr (closes #230, closes #234)
- Add since/until/decider to list_adrs, status/case_sensitive/snippets to search_adrs, run_doctor and export_adrs tools (closes #231, closes #232, closes #233)

### Testing

- Add tests for string-or-vec parsing and doctor parse error reporting
- Add MCP server tests using in-process ChannelTransport client
- Add missing tests for template, export, cli, search, lint, config (closes #235, closes #236, closes #237, closes #238, closes #239, closes #241)


## [0.7.3] - 2026-03-04

### Bug Fixes

- Open actual ADR file in editor instead of temp file


## [0.7.2] - 2026-02-26

### Documentation

- Fix documentation link and update README
- Audit and fix book documentation


## [0.7.1] - 2026-02-24

### Bug Fixes

- Preserve file content when updating ADR metadata ([#187](https://github.com/joshrotenberg/adrs/pull/187))


## [0.7.0] - 2026-02-20

### Bug Fixes

- Generate functional supersedes/superseded-by markdown links
- Honor config mode=ng for tags in `new` command ([#181](https://github.com/joshrotenberg/adrs/pull/181))

### Styling

- Run cargo fmt
- Run cargo fmt


## [0.6.2] - 2026-02-11

### Bug Fixes

- Honor adrs.toml template fields and accept "nextgen" mode
- Wire custom template config, validate empty adr_dir, add tests
- Resolve security audit failures and update dependencies

### Testing

- Add scenario test for config-driven template selection


## [0.6.1] - 2026-01-27

### Bug Fixes

- MADR format ADRs not parsed correctly
- Persist tags in ADR YAML frontmatter

### Features

- Enable MCP server by default

### Testing

- Add comprehensive smoke tests for CLI


## [0.6.0] - 2026-01-26

### Bug Fixes

- Init detects existing ADRs and skips initial ADR creation
- Expose MCP tools via ServerHandler list_tools/call_tool

### Documentation

- Improve CLI help discoverability with examples
- Add missing documentation for v0.6.0 features

### Features

- Add source_uri field to JSON-ADR spec for federation
- Add filtering options to list command
- Add search command for full-text search
- Add template management commands
- Add tags support for ADR categorization
- Add --no-edit flag for non-interactive ADR creation
- Simplify link command with auto-derived reverse links
- Add shell completions command
- Add MCP server for AI agent integration
- Add MCP write operations (create, status, link)
- Add HTTP transport for MCP server
- Add MCP tools for content updates and repository info
- **mcp:** Add validate_adr tool
- Add cheatsheet command for quick reference
- **mcp:** Add get_adr_sections, compare_adrs, bulk_update_status, suggest_tags tools

### Testing

- Add integration tests for new CLI commands


## [0.5.1] - 2026-01-22

### Documentation

- Refresh README for v0.5.0

### Features

- Add status command to change ADR status

### Testing

- Add integration tests for status command


## [0.5.0] - 2026-01-22

### Bug Fixes

- Add version to adrs-core dependency for cargo publish
- Use workspace dependency for adrs-core

### Documentation

- Rewrite book and improve CLI help for v2

### Features

- V2 rewrite with library-first architecture
- Add MADR 4.0.0 support
- Add template variants (full, minimal, bare)
- Add doctor command for repository health checks
- Add config discovery with directory tree search

### Testing

- Add CLI integration tests
- Add scenario tests for end-to-end workflows

