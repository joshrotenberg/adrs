# 8. Use tower-mcp for MCP server

Date: 2026-01-28

## Status

Accepted

## Context

The adrs MCP server was originally implemented using the `rmcp` crate, which provides a low-level Rust SDK for the Model Context Protocol. While functional, `rmcp` requires manual implementation of the `ServerHandler` trait, explicit tool registration via `list_tools`/`call_tool` methods, and hand-wired JSON schema generation for tool parameters.

The `tower-mcp` crate (v0.1) builds on top of `rmcp` and the Tower service framework to provide a higher-level, more ergonomic API. It offers a `McpRouter` with a builder pattern, `ToolBuilder` for declarative tool definitions, built-in support for both stdio and HTTP transports, and automatic parameter schema derivation via `schemars`.

As the MCP server grew from a handful of tools to 15 tools with complex parameter types, the boilerplate required by raw `rmcp` became a maintenance burden.

## Decision

Migrate the MCP server implementation from `rmcp` to `tower-mcp`.

Key aspects of the migration:

1. **Router-based architecture**: Replace the manual `ServerHandler` trait implementation with `McpRouter::new()` and chained `.tool()` calls.

2. **Declarative tool definitions**: Use `ToolBuilder` to define each tool with name, description, and typed async handler, replacing the manual `list_tools`/`call_tool` dispatch.

3. **Transport abstraction**: Use `tower-mcp`'s built-in `StdioTransport` and `HttpTransport` instead of manually wiring transports.

4. **Macro for boilerplate reduction**: Introduce an `adr_tool!` macro to further reduce repetitive tool registration code across the 15 MCP tools.

5. **Workspace dependency**: Declare `tower-mcp` as a workspace dependency with optional `http` feature for HTTP transport support.

## Consequences

- Significantly reduced boilerplate: tool registration is declarative rather than imperative
- Adding new tools requires only a `ToolBuilder` call and handler function, not modifying a central dispatch match
- Transport setup is handled by the framework rather than custom code
- Takes a dependency on `tower-mcp` (which itself depends on `rmcp` and `tower`), adding to the dependency tree
- The `tower-mcp` crate is at v0.1, so API stability is not yet guaranteed
- HTTP transport support comes for free via a feature flag rather than custom implementation