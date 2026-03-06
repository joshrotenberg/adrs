# 12. MCP server library selection

Date: 2026-03-04

## Status

Proposed

## Context

[ADR-0011](./0011-mcp-server-integration.md) established that the adrs CLI should include MCP server functionality via an optional feature flag, using the `rmcp` crate. As the MCP server has grown to 15 tools, a maintenance burden has emerged with manual `ServerHandler` trait implementation and explicit tool dispatch.

[PR #171](https://github.com/joshrotenberg/adrs/pull/171) proposes migrating from `rmcp` to `tower-mcp`, claiming reduced boilerplate. This ADR evaluates whether that migration is warranted and surveys the broader Rust MCP library ecosystem.

### Research Questions

1. Is `tower-mcp` worth it over `rmcp`?
2. What do we lose by switching?
3. What do we gain by switching?
4. What alternative libraries exist?
5. Do any support MCP 2025-11-25 Streamable HTTP transport?

### MCP Protocol Version Context

The [MCP 2025-11-25 specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports) defines two standard transports:

1. **stdio**: Client spawns server as subprocess, communicates via stdin/stdout
2. **Streamable HTTP**: HTTP POST/GET with optional Server-Sent Events (SSE) for streaming

Streamable HTTP replaces the deprecated HTTP+SSE transport from 2024-11-05 and adds:
- Session management via `Mcp-Session-Id` header
- Resumability through SSE event IDs
- Multi-client concurrent connections

## Options Evaluated

### 1. rmcp (Official SDK)

**Repository:** [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
**Version:** 1.1.0 (current), 0.14.0 (in Cargo.lock)
**License:** Apache-2.0

**Transport Support:**
- stdio (via `transport-io`)
- Streamable HTTP server (via `transport-streamable-http-server`)
- Streamable HTTP client (via `transport-streamable-http-client`)
- Tower integration (via `tower` feature)

**Architecture:**
- Proc macro based: `#[tool]`, `#[tool_router]`, `#[prompt_router]`
- Manual `ServerHandler` trait implementation
- Transport setup requires explicit wiring

**Pros:**
- Official MCP SDK, canonical implementation
- Comprehensive transport support
- Active maintenance by MCP team
- Mature and battle-tested

**Cons:**
- Boilerplate for `list_tools`/`call_tool` dispatch
- Manual JSON schema generation setup
- Less ergonomic API than alternatives

### 2. tower-mcp

**Repository:** [joshrotenberg/tower-mcp](https://github.com/joshrotenberg/tower-mcp)
**Version:** 0.7.0
**License:** MIT OR Apache-2.0
**Note:** Maintained by same author as this `adrs` repository

**Transport Support:**
- stdio
- HTTP with SSE and stream resumption
- WebSocket (feature-gated)
- Child process spawning

**Architecture:**
- Builder pattern API (no proc macros required)
- Axum-style extractors (`State`, `Context`, `Json`)
- Tower middleware composition via `.layer()`
- `McpRouter` + `ToolBuilder` pattern

**Pros:**
- Declarative tool registration (builder pattern)
- Native Tower middleware support
- Familiar axum-style ergonomics
- 39/39 MCP conformance tests passing
- Simpler transport abstraction

**Cons:**
- Maintained by single developer (conflict of interest in PR)
- Heavier dependency tree (axum, hyper, tower)
- Younger project (v0.7.0, API may evolve)
- Requires Tower/Service familiarity

### 3. rust-mcp-sdk

**Repository:** [rust-mcp-stack/rust-mcp-sdk](https://github.com/rust-mcp-stack/rust-mcp-sdk)
**Version:** 0.8.x
**License:** MIT

**Transport Support:**
- stdio
- Streamable HTTP (server-side)
- SSE (backward compatible)
- Hyper-based multi-client server

**Architecture:**
- Proc macros: `mcp_tool`, `tool_box`, `mcp_elicit`
- Dual handler traits: `ServerHandler`/`ServerHandlerCore`
- Built-in OAuth providers (Keycloak, WorkOS, Scalekit)
- Event store for resumability

**Pros:**
- Enterprise features (OAuth, multi-tenant)
- MCP 2025-11-25 compliant
- Built-in session management
- DNS rebinding protection

**Cons:**
- Heavy dependency footprint
- Under active development (breaking changes)
- Overkill for simple use cases

### 4. mcpkit

**Repository:** [praxiomlabs/mcpkit](https://github.com/praxiomlabs/mcpkit)
**Version:** Recent
**License:** MIT

**Transport Support:**
- stdio
- HTTP (Streamable HTTP)
- WebSocket
- gRPC
- Unix sockets / Windows named pipes

**Architecture:**
- Unified `#[mcp_server]` macro
- Runtime-agnostic (Tokio, smol, others)
- Tower-compatible middleware via Layer pattern
- Typestate builders for compile-time validation

**Pros:**
- Most comprehensive transport support
- MCP 2025-11-25 compliant (newest spec)
- Runtime-agnostic design
- Supports Tasks, Elicitation, OAuth 2.1

**Cons:**
- Newer project
- More complex than needed for simple servers

### 5. turbomcp

**Repository:** [epistates/turbomcp](https://github.com/epistates/turbomcp)
**Version:** Pre-release
**License:** MIT

**Transport Support:**
- stdio, HTTP/SSE (MCP standard)
- WebSocket, TCP, Unix sockets (extensions)

**Architecture:**
- Zero-boilerplate proc macros
- SIMD-accelerated JSON (simd-json, sonic-rs)
- Type-state capability builders
- Built-in benchmarking and security auditing

**Pros:**
- Performance optimized (16x faster than TypeScript)
- Enterprise security features (OAuth 2.1, rate limiting, CORS)
- MCP 2025-11-25 compliant
- Comprehensive examples (26+)

**Cons:**
- Enterprise complexity for simple use case
- Pre-release maturity
- Large dependency surface

### 6. kuri

**Repository:** [itsaphel/kuri](https://github.com/itsaphel/kuri)
**Version:** 0.2.0
**License:** MIT

**Transport Support:**
- stdio (implemented)
- Streamable HTTP (planned)

**Architecture:**
- Minimal macros (`#[tool]`, `#[prompt]`)
- Tower ecosystem integration
- Ergonomic design philosophy

**Pros:**
- Simplest API surface
- Tower middleware reuse
- "MCP should feel like normal Rust"

**Cons:**
- Early stage (v0.2.0)
- HTTP transport not yet implemented
- Limited feature set

## Comparison Matrix

| Library | Version | Transports | MCP Spec | Architecture | Complexity |
|---------|---------|------------|----------|--------------|------------|
| **rmcp** | 1.1.0 | stdio, HTTP, SSE | 2025-11-25 | Proc macros + trait | Medium |
| **tower-mcp** | 0.7.0 | stdio, HTTP, WS | 2025-11-25 | Builder + Tower | Medium |
| **rust-mcp-sdk** | 0.8.x | stdio, HTTP, SSE | 2025-11-25 | Proc macros + OAuth | High |
| **mcpkit** | recent | stdio, HTTP, WS, gRPC, Unix | 2025-11-25 | Unified macro | Medium |
| **turbomcp** | pre | stdio, HTTP, WS, TCP | 2025-11-25 | Zero-boilerplate macros | High |
| **kuri** | 0.2.0 | stdio (HTTP planned) | partial | Minimal macros + Tower | Low |

## Analysis

### Question 1: Is tower-mcp worth it over rmcp?

**Partially.** `tower-mcp` offers ergonomic improvements:
- Builder pattern reduces boilerplate vs manual trait impl
- Tower middleware enables cross-cutting concerns cleanly
- Axum-style extractors are familiar to Rust web developers

However, concerns exist:
- **Conflict of interest**: The PR author (joshrotenberg) maintains both the `adrs` repo and `tower-mcp`
- **Version discrepancy**: PR references v0.1.0, current is v0.7.0 (6 breaking versions)
- **rmcp has improved**: v1.1.0 adds Tower integration via `tower` feature

### Question 2: What do we lose?

- **Official SDK status**: `rmcp` is the canonical implementation
- **Stability**: `rmcp` is more mature
- **Community**: Larger user base for troubleshooting
- **Direct MCP team support**: Issues go directly to protocol authors

### Question 3: What do we gain?

- **Reduced boilerplate**: ~50 fewer lines per tool registration
- **Middleware composition**: Standard Tower `.layer()` pattern
- **Declarative tools**: ToolBuilder vs manual dispatch
- **Familiar patterns**: Axum users will feel at home

### Question 4: What alternatives exist?

See Options Evaluated above. Notable alternatives:
- **mcpkit**: Most complete transport support, MCP 2025-11-25
- **turbomcp**: Performance-focused enterprise solution
- **kuri**: Simplest API but limited transports

### Question 5: Modern MCP spec support?

All major libraries support MCP 2025-11-25 Streamable HTTP:
- **rmcp**: Yes (v1.1.0, `transport-streamable-http-server`)
- **tower-mcp**: Yes (HTTP + SSE with resumption)
- **rust-mcp-sdk**: Yes (native Streamable HTTP)
- **mcpkit**: Yes (plus gRPC, WebSocket)
- **turbomcp**: Yes (plus WebSocket, TCP)

## Decision

**Defer migration.** The analysis reveals:

1. **PR is outdated**: References tower-mcp v0.1.0; current is v0.7.0
2. **rmcp has evolved**: v1.1.0 includes Tower integration
3. **Conflict of interest**: PR author maintains tower-mcp
4. **Low urgency**: Current implementation works; boilerplate is manageable

### Recommended Path Forward

1. **Update rmcp to v1.1.0** and evaluate its new Tower features
2. **Re-evaluate** once tower-mcp reaches v1.0 stable
3. **Consider mcpkit** if additional transports (gRPC, WebSocket) are needed
4. **Document criteria** for future library changes

### If Migration Proceeds

Should we decide to migrate in the future, prefer:
1. **mcpkit** for comprehensive transport support
2. **tower-mcp** for minimal change from current patterns
3. **turbomcp** if performance becomes critical

## Consequences

### If We Stay with rmcp

- Continue using current patterns
- Upgrade to v1.1.0 for Tower integration option
- Maintain compatibility with official SDK
- Lower risk, predictable maintenance

### If We Migrate Later

- Document clear migration criteria
- Ensure thorough test coverage before migration
- Consider API stability of target library
- Evaluate transport requirements (HTTP, WebSocket, etc.)

## References

- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
- [rmcp GitHub](https://github.com/modelcontextprotocol/rust-sdk)
- [tower-mcp GitHub](https://github.com/joshrotenberg/tower-mcp)
- [PR #171](https://github.com/joshrotenberg/adrs/pull/171)
- [Shuttle MCP Comparison](https://www.shuttle.dev/blog/2025/09/15/mcp-servers-rust-comparison)
- [Build Streamable HTTP MCP](https://www.shuttle.dev/blog/2025/10/29/stream-http-mcp)
