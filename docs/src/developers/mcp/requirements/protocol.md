# Protocol Requirements

<!-- toc -->

## Transport

### MCP-PROTO-1: Transport Support

**Requirements:**
- MUST support stdio transport (default)
- SHOULD support HTTP transport (optional feature)
- MUST implement MCP JSON-RPC protocol

### MCP-PROTO-2: Stdio Transport

**Requirements:**
- MUST read from stdin
- MUST write to stdout
- MUST use stderr for logging only
- MUST handle EOF gracefully

### MCP-PROTO-3: HTTP Transport

**Requirements:**
- MUST listen on configurable address
- MUST support `/mcp` endpoint
- SHOULD support health check endpoint
- MUST handle concurrent requests

## Protocol

### MCP-PROTO-4: JSON-RPC

**Requirements:**
- MUST implement JSON-RPC 2.0
- MUST handle batch requests
- MUST return proper error codes
- MUST include request IDs in responses

### MCP-PROTO-5: Tool Registration

**Requirements:**
- All tools MUST be registered with name
- All tools MUST include description
- All tools MUST have input schema (JSON Schema)
- Schemas MUST be valid JSON Schema

### MCP-PROTO-6: Error Responses

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid request",
    "data": { "details": "..." }
  }
}
```

**Standard Codes:**
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## See Also

- [Tool Requirements](./tools.md)
- [MCP Specification](https://modelcontextprotocol.io/)
