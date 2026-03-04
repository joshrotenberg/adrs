# Developer Guide

Welcome to the `adrs` developer guide. This section is for developers who want to:

- Extend the `adrs` CLI
- Consume the `adrs-core` library
- Contribute to the project

## Sections

### [Introduction](./introduction/README.md)

Overview of the `adrs` architecture and codebase.

### [Library (adrs-core)](./lib/README.md)

Using the `adrs-core` Rust library in your own projects.

### [CLI (adrs)](./cli/README.md)

Architecture and extension points for the CLI.

### [MCP Server](./mcp/README.md)

The Model Context Protocol server for AI agent integration.

### [Testing](./testing/README.md)

Testing strategy and guidelines.

### [Contributing](./contributing.md)

How to contribute to the project.

## Quick Links

- [GitHub Repository](https://github.com/joshrotenberg/adrs)
- [API Documentation](https://docs.rs/adrs-core)
- [Crates.io](https://crates.io/crates/adrs)

## Architecture Overview

```
adrs (workspace)
├── crates/
│   ├── adrs-core/     # Core library (parsing, templates, repository)
│   ├── adrs-cli/      # CLI application
│   └── adrs-mcp/      # MCP server
└── docs/              # Documentation (mdBook)
```

The project follows a library-first architecture where all core functionality lives in `adrs-core`, with thin wrappers for the CLI and MCP server.

## Requirements

- [Project Requirements](./requirements/README.md)
- [Library Requirements](./lib/requirements/README.md)
- [CLI Requirements](./cli/requirements/README.md)
- [MCP Requirements](./mcp/requirements/README.md)
