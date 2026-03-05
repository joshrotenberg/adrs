# MCP Usage Examples

Examples of using `adrs` MCP tools with AI agents.

## Conversation Examples

### List ADRs

> **User**: "What architectural decisions have been made in this project?"

The AI uses `list_adrs` to retrieve and summarize ADRs.

### Search for Topics

> **User**: "Are there any decisions about authentication?"

The AI uses `search_adrs` with "authentication" to find relevant ADRs.

### Create ADR

> **User**: "Create an ADR for using Redis as our caching layer"

The AI uses `create_adr` with the title and may populate initial content.

### Review and Accept

> **User**: "Accept ADRs 5, 6, and 7"

The AI uses `bulk_update_status` to accept multiple ADRs.

### Compare Decisions

> **User**: "Compare ADR 2 with ADR 8 to see how our database choice evolved"

The AI uses `compare_adrs` to show differences.

## Tool Call Examples

### list_adrs

```json
{
  "name": "list_adrs",
  "arguments": {
    "status": "proposed"
  }
}
```

Response:
```json
[
  {
    "number": 5,
    "title": "Use Redis for caching",
    "status": "proposed",
    "date": "2024-01-15"
  }
]
```

### create_adr

```json
{
  "name": "create_adr",
  "arguments": {
    "title": "Use Redis for caching",
    "context": "We need a caching layer to improve performance.",
    "decision": "We will use Redis as our caching solution.",
    "consequences": "We need to manage Redis infrastructure."
  }
}
```

### link_adrs

```json
{
  "name": "link_adrs",
  "arguments": {
    "source": 5,
    "target": 2,
    "link_type": "Amends"
  }
}
```

### validate_adr

```json
{
  "name": "validate_adr",
  "arguments": {
    "number": 5
  }
}
```

Response:
```json
{
  "valid": false,
  "issues": [
    {
      "severity": "warning",
      "message": "Context section is empty"
    }
  ]
}
```

## See Also

- [Tools Reference](../tools/README.md)
- [Adding Tools](./add-tools.md)
