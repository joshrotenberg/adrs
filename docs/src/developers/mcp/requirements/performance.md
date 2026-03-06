# Performance Requirements

<!-- toc -->

## Response Times

### MCP-PERF-1: Read Operations

| Tool | Target | Maximum |
|------|--------|---------|
| `list_adrs` | < 50ms | < 100ms |
| `get_adr` | < 20ms | < 100ms |
| `search_adrs` | < 100ms | < 500ms |
| `get_adr_sections` | < 30ms | < 100ms |
| `get_related_adrs` | < 50ms | < 200ms |
| `get_repository_info` | < 10ms | < 50ms |

### MCP-PERF-2: Write Operations

| Tool | Target | Maximum |
|------|--------|---------|
| `create_adr` | < 100ms | < 500ms |
| `update_status` | < 100ms | < 500ms |
| `link_adrs` | < 200ms | < 500ms |
| `update_content` | < 100ms | < 500ms |
| `update_tags` | < 50ms | < 200ms |

### MCP-PERF-3: Bulk Operations

| Tool | Target | Maximum |
|------|--------|---------|
| `bulk_update_status` (10 items) | < 500ms | < 2s |
| `bulk_update_status` (100 items) | < 2s | < 10s |

### MCP-PERF-4: Analysis Operations

| Tool | Target | Maximum |
|------|--------|---------|
| `validate_adr` | < 50ms | < 200ms |
| `compare_adrs` | < 100ms | < 300ms |
| `suggest_tags` | < 200ms | < 500ms |

## Scalability

### MCP-PERF-5: Repository Size

**Requirements:**
- MUST handle repositories with 1000+ ADRs
- List operations MUST NOT degrade linearly with size
- Search SHOULD use indexing for large repositories

### MCP-PERF-6: Concurrent Requests

**Requirements:**
- MUST handle multiple concurrent requests
- Read operations MUST NOT block other reads
- Write operations MAY serialize

## Resource Usage

### MCP-PERF-7: Memory

**Requirements:**
- Idle memory usage SHOULD be < 50MB
- Per-request memory SHOULD be < 10MB
- MUST NOT hold entire repository in memory

### MCP-PERF-8: Startup

**Requirements:**
- Cold start SHOULD be < 500ms
- Tool registration SHOULD be < 100ms

## Measurement

### How to Measure

```rust
use std::time::Instant;

let start = Instant::now();
let result = handle_request(req).await;
let duration = start.elapsed();

metrics::histogram!("mcp.request.duration", duration);
```

### Benchmarks

```sh
# Run benchmarks
cargo bench -p adrs-mcp
```

## See Also

- [Tool Requirements](./tools.md)
- [Safety Requirements](./safety.md)
