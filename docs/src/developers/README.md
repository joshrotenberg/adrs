# Developer Guide

Welcome to the `adrs` developer guide. This section is for developers who want to:

- Extend the `adrs` CLI
- Consume the `adrs-core` library
- Contribute to the project

## Quick Links

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

- [Project Requirements](../requirements/project/README.md)
- [Library Requirements](./lib/requirements/README.md)
- [CLI Requirements](./cli/requirements/README.md)
- [MCP Requirements](./mcp/requirements/README.md)
