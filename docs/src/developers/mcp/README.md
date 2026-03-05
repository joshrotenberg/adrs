# MCP Server

`adrs` includes an MCP (Model Context Protocol) server for AI agent integration.

## Installation

The MCP server is included by default. A plain `cargo install adrs` includes it.

For HTTP transport support:

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

## Best Practices

1. **Human Review**: All ADRs created by AI are set to 'proposed' status
2. **Provide Context**: Give AI context about your architecture
3. **Use Tags**: In NextGen mode, use `suggest_tags` for consistent categorization
4. **Validate Often**: Use `validate_adr` to ensure complete content
5. **Link Related**: Use `link_adrs` to maintain traceability

## Documentation

- [Tools Reference](./tools/README.md) - Available MCP tools
- [Examples](./examples/usage.md) - Usage examples
- [Requirements](./requirements/README.md) - Design requirements

## See Also

- [AI Documentation](../../ai/README.md) - AI agent docs
- [Configuration](../../users/configuration.md) - Repository configuration
- [Modes](../../users/modes/README.md) - NextGen vs Compatible mode
