# MCP Server

`adrs` includes an MCP (Model Context Protocol) server for AI agent integration. This allows AI assistants like Claude to read, search, create, and manage ADRs directly.

## Installation

The MCP server is included by default since v0.6.1. A plain `cargo install adrs` includes it.

For HTTP transport support (independent server mode):

```sh
cargo install adrs --features mcp-http
```

## Configuration

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "adrs": {
      "command": "adrs",
      "args": ["mcp", "serve"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

### HTTP Mode

For independent server operation:

```sh
adrs mcp serve --http 127.0.0.1:3000
```

Then configure Claude with:

```json
{
  "mcpServers": {
    "adrs": {
      "url": "http://127.0.0.1:3000/mcp"
    }
  }
}
```

## Available Tools

The MCP server provides 17 tools organized by function:

### Read Operations

| Tool | Description |
|------|-------------|
| `list_adrs` | List all ADRs with optional status/tag/date/decider filters |
| `get_adr` | Get full content of an ADR by number |
| `search_adrs` | Search ADR titles and content with optional status filter and case sensitivity |
| `get_adr_sections` | Get ADR with parsed sections (context, decision, consequences) |
| `get_related_adrs` | Get all ADRs linked to/from a specific ADR |
| `get_repository_info` | Get repository mode, ADR count, and configuration |
| `run_doctor` | Check repository health: broken links, parse errors, duplicate numbers |
| `export_adrs` | Export ADRs to JSON-ADR format with optional filtering |

### Write Operations

| Tool | Description |
|------|-------------|
| `create_adr` | Create a new ADR (always 'proposed' status) |
| `update_status` | Change an ADR's status |
| `link_adrs` | Create bidirectional links between ADRs |
| `update_content` | Update ADR sections (context, decision, consequences) |
| `update_tags` | Add or replace tags (NextGen mode only) |
| `bulk_update_status` | Update multiple ADR statuses at once |

### Analysis Tools

| Tool | Description |
|------|-------------|
| `validate_adr` | Check ADR structure and report issues |
| `compare_adrs` | Compare two ADRs and show differences |
| `suggest_tags` | Analyze content and suggest relevant tags |

## Tool Details

### list_adrs

List all ADRs with optional filtering.

**Parameters:**
- `status` (optional): Filter by status (proposed, accepted, deprecated, superseded)
- `tag` (optional): Filter by tag (NextGen mode)
- `since` (optional): Filter ADRs on or after this date (YYYY-MM-DD, inclusive)
- `until` (optional): Filter ADRs on or before this date (YYYY-MM-DD, inclusive)
- `decider` (optional): Filter by decider name -- case-insensitive substring match against decision_makers

**Example response:**
```json
[
  {
    "number": 1,
    "title": "Use PostgreSQL for persistence",
    "status": "accepted",
    "date": "2026-01-15",
    "tags": ["database", "infrastructure"]
  }
]
```

### get_adr

Get the full content of a specific ADR.

**Parameters:**
- `number` (required): The ADR number

**Returns:** Complete ADR including title, status, content, and links.

### search_adrs

Search ADRs for matching text.

**Parameters:**
- `query` (required): Search text
- `title_only` (optional): Only search titles (default: false)
- `status` (optional): Filter results by ADR status
- `case_sensitive` (optional): Perform case-sensitive search (default: false)

**Returns:** List of matching ADRs with match snippets showing which section matched and context around the match.

### create_adr

Create a new ADR. Always creates with 'proposed' status for human review.

**Parameters:**
- `title` (required): ADR title
- `context` (optional): Context section content
- `decision` (optional): Decision section content
- `consequences` (optional): Consequences section content
- `supersedes` (optional): ADR number this supersedes
- `format` (optional): Template format -- `nygard` (default) or `madr` for MADR 4.0.0 format
- `variant` (optional): Template variant -- `full` (default), `minimal`, or `bare`
- `decision_makers` (optional): List of people who made the decision (most useful with MADR format)
- `consulted` (optional): List of people consulted for input (most useful with MADR format)
- `informed` (optional): List of people kept informed (most useful with MADR format)
- `tags` (optional): Tags for categorization. Applied at creation in NextGen mode. In compatible mode a warning is returned in the response instead of an error.

### update_status

Change an ADR's status.

**Parameters:**
- `number` (required): ADR number
- `status` (required): New status (proposed, accepted, deprecated, superseded, rejected)
- `superseded_by` (optional): Required when status is 'superseded'

### link_adrs

Create bidirectional links between ADRs.

**Parameters:**
- `source` (required): Source ADR number
- `target` (required): Target ADR number
- `link_type` (required): Link type (Supersedes, Amends, "Relates to")

### validate_adr

Validate an ADR's structure and content.

**Parameters:**
- `number` (required): ADR number to validate

**Returns:** Validation results with severity levels (error/warning).

### compare_adrs

Compare two ADRs side by side.

**Parameters:**
- `source` (required): First ADR number
- `target` (required): Second ADR number

**Returns:** Structural comparison of title, status, and content sections.

### suggest_tags

Analyze ADR content and suggest relevant tags.

**Parameters:**
- `number` (required): ADR number to analyze

**Returns:** Suggested tags with confidence scores and reasons.

### run_doctor

Run health checks on the ADR repository.

**Parameters:** None

**Returns:** Health report with issue counts and details. Each issue includes severity (error/warning/info), rule ID, rule name, message, and location.

### export_adrs

Export ADRs to JSON-ADR format.

**Parameters:**
- `numbers` (optional): Array of ADR numbers to export (exports all if omitted)
- `metadata_only` (optional): If true, exclude content sections from output (default: false)

**Returns:** JSON-ADR format export with schema, version, and ADR array.

## Usage Examples

### Ask Claude to list your ADRs

> "What architectural decisions have been made in this project?"

Claude will use `list_adrs` to retrieve and summarize your ADRs.

### Search for specific topics

> "Are there any decisions about authentication?"

Claude will use `search_adrs` with "authentication" to find relevant ADRs.

### Create a new ADR

> "Create an ADR for using Redis as our caching layer"

Claude will use `create_adr` with the title and may populate initial content based on your discussion.

### Review and accept ADRs

> "Accept ADRs 5, 6, and 7"

Claude will use `bulk_update_status` to accept multiple ADRs.

### Analyze decisions

> "Compare ADR 2 with ADR 8 to see how our database choice evolved"

Claude will use `compare_adrs` to show the differences.

## Best Practices

1. **Human Review**: All ADRs created by AI are set to 'proposed' status. Review before accepting.

2. **Provide Context**: Give Claude context about your architecture when asking it to create ADRs.

3. **Use Tags**: In NextGen mode, use `suggest_tags` to maintain consistent categorization.

4. **Validate Often**: Use `validate_adr` to ensure ADRs have complete content.

5. **Link Related Decisions**: Use `link_adrs` to maintain decision traceability.

## See Also

- [Configuration](./configuration.md) - Repository configuration
- [Formats](./formats.md) - ADR format options
- [Compatibility](./compatibility.md) - NextGen vs Compatible mode
