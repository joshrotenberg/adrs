# MCP Server Requirements

Requirements for the `adrs-mcp` MCP (Model Context Protocol) server.

## Protocol Requirements

### MCP-1: Transport

- MUST support stdio transport (default)
- SHOULD support HTTP transport (optional feature)
- MUST implement MCP JSON-RPC protocol

### MCP-2: Tool Registration

All tools MUST be registered with:
- Name
- Description
- Input schema (JSON Schema)

## Tool Requirements

### MCP-3: Read Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_adrs` | List all ADRs | status?, tag? |
| `get_adr` | Get single ADR | number |
| `search_adrs` | Search ADRs | query, title_only? |
| `get_adr_sections` | Get parsed sections | number |
| `get_related_adrs` | Get linked ADRs | number |
| `get_repository_info` | Repository metadata | - |

### MCP-4: Write Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `create_adr` | Create ADR | title, context?, decision?, consequences?, supersedes? |
| `update_status` | Change status | number, status, superseded_by? |
| `link_adrs` | Create link | source, target, link_type |
| `update_content` | Update sections | number, context?, decision?, consequences? |
| `update_tags` | Set tags | number, tags |
| `bulk_update_status` | Batch status change | updates[] |

### MCP-5: Analysis Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `validate_adr` | Check structure | number |
| `compare_adrs` | Compare two ADRs | adr1, adr2 |
| `suggest_tags` | Suggest tags | number |

## Safety Requirements

### MCP-6: Safe Defaults

- `create_adr` MUST create with `proposed` status
- Write operations MUST NOT delete files
- MUST validate inputs before operations

### MCP-7: Error Handling

- MUST return structured errors
- MUST include error codes
- MUST NOT expose internal paths

## Performance

### MCP-8: Response Time

- Read operations: < 100ms
- Write operations: < 500ms
- Bulk operations: < 2s for 100 items
