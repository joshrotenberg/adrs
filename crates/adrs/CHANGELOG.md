# Changelog

All notable changes to this project will be documented in this file.

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

