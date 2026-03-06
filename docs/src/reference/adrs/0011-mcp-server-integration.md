# 11. MCP server integration

Date: 2025-03-04

## Status

Proposed

## Context

AI assistants (Claude, GPT, etc.) benefit from direct access to ADR repositories:

- Reading ADRs for context when discussing decisions
- Creating new ADRs based on conversations
- Updating ADR status as decisions evolve
- Searching and linking related ADRs

The Model Context Protocol (MCP) provides a standard for AI tool integration.

Options considered:

1. **Separate MCP server binary**: Independent crate/binary
2. **CLI feature flag**: Optional feature in existing CLI
3. **Library-only**: Let consumers build their own MCP server

## Decision

Implement MCP server as an optional feature (`mcp`) in the CLI crate because:

- Avoids additional binary/crate complexity
- Reuses existing CLI infrastructure
- Can be disabled for minimal builds
- Library-first architecture means all logic is in `adrs-core`

The MCP server provides 15 tools:

**Read Operations:**
- `list_adrs`: List with filters (status, tags)
- `get_adr`: Full content by number
- `search_adrs`: Search titles and content
- `get_adr_sections`: Parsed sections
- `get_related_adrs`: Linked ADRs
- `get_repository_info`: Configuration

**Write Operations:**
- `create_adr`: Create new ADR
- `update_status`: Change status
- `link_adrs`: Create bidirectional links
- `update_content`: Update sections
- `update_tags`: Manage tags
- `bulk_update_status`: Batch updates

**Analysis Tools:**
- `validate_adr`: Check structure
- `compare_adrs`: Compare two ADRs
- `suggest_tags`: Tag suggestions

## Consequences

- AI assistants can manage ADRs directly
- MCP is optional (compile-time feature)
- Adds `rmcp` dependency when enabled
- Server requires configured ADR repository
- Write operations modify files (appropriate warnings to users)
