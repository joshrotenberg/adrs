# AI Agent Documentation

This section provides documentation specifically for AI agents working with `adrs`.

## Overview

`adrs` includes an MCP (Model Context Protocol) server that enables AI agents to:
- Read and search ADRs
- Create and update ADRs
- Analyze and validate decisions
- Track decision relationships

## Sections

- [Skills](./skills/README.md) - Pre-defined agent capabilities
- [Rules](./rules/README.md) - Behavioral guidelines for AI agents
- [Hooks](./hooks/README.md) - Integration hooks and triggers

## MCP Integration

AI agents connect via MCP server:

```json
{
  "mcpServers": {
    "adrs": {
      "command": "adrs",
      "args": ["mcp", "serve"],
      "cwd": "/path/to/project"
    }
  }
}
```

## Available Tools

| Category | Tools |
|----------|-------|
| Read | `list_adrs`, `get_adr`, `search_adrs`, `get_adr_sections`, `get_related_adrs`, `get_repository_info` |
| Write | `create_adr`, `update_status`, `link_adrs`, `update_content`, `update_tags`, `bulk_update_status` |
| Analysis | `validate_adr`, `compare_adrs`, `suggest_tags` |

## Best Practices

1. **Always validate before modifying**: Use `validate_adr` before updates
2. **Create with proposed status**: New ADRs should be proposed, not accepted
3. **Link related decisions**: Use `link_adrs` to maintain traceability
4. **Search before creating**: Check for existing related decisions
5. **Use structured content**: Follow format conventions for consistency

## See Also

- [MCP Server](../developers/mcp/README.md) - Developer documentation
- [Tools Reference](../developers/mcp/tools/README.md) - Tool specifications
