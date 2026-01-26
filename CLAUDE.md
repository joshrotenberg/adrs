# adrs - Architecture Decision Records CLI

## Project Overview

A Rust CLI tool for managing Architecture Decision Records (ADRs). Compatible with adr-tools format while adding modern features.

## Key Commands

```bash
adrs init                    # Initialize ADR repository
adrs new "Title"             # Create new ADR
adrs list                    # List all ADRs
adrs search "query"          # Search ADRs
adrs link 2 1 supersedes     # Link ADRs
adrs status 1 accepted       # Change status
adrs export json             # Export to JSON-ADR format
adrs import file.json        # Import from JSON-ADR
adrs generate toc            # Generate table of contents
adrs generate graph          # Generate Graphviz diagram
adrs doctor                  # Check repository health
adrs mcp serve               # Start MCP server (requires --features mcp)
```

## Architecture

- **adrs-core**: Library crate with core types (Adr, Repository, Config)
- **adrs**: CLI binary using clap, optional MCP server

## Modes

- **Compatible**: adr-tools format (status in markdown body)
- **NextGen**: YAML frontmatter with extended fields (tags, links, MADR fields)

## MCP Server

Optional feature for AI agent integration. Build with:
```bash
cargo build --release --features mcp
```

Tools provided:
- `list_adrs` - List ADRs with optional filters
- `get_adr` - Get full ADR content by number
- `search_adrs` - Search ADR titles and content
- `create_adr` - Create new ADR (always 'proposed' status)
- `update_status` - Change ADR status
- `link_adrs` - Create bidirectional links

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --all-features
cargo test --test '*' --all-features
```

## Current Work

- Issue #135: ADRs in the age of AI agents (research/design)
- PR #137: MCP server (merged)
- Exploring: CI enforcement, review tools, drift detection

## Related Issues

- #94: MCP server feature
- #135: AI agent integration philosophy
