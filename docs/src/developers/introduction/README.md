# Developer Introduction

This guide provides an overview of the `adrs` project for developers.

## Development Setup

### Prerequisites

- Rust toolchain (stable, 1.70+)
- Git

### Getting Started

```sh
# Clone the repository
git clone https://github.com/joshrotenberg/adrs
cd adrs

# Build
cargo build

# Run tests
cargo test --all

# Run clippy
cargo clippy --all-targets

# Build docs
mdbook build docs
```

### Running Locally

```sh
# Run CLI
cargo run -- --help
cargo run -- list

# Run with features
cargo run --features mcp-http -- mcp serve --http 127.0.0.1:3000
```

## Project Structure

```
adrs/
├── crates/
│   ├── adrs-core/        # Core library
│   │   ├── src/
│   │   │   ├── config.rs       # Configuration handling
│   │   │   ├── repository.rs   # Repository operations
│   │   │   ├── template.rs     # Template engine
│   │   │   ├── parse.rs        # ADR file parsing
│   │   │   ├── types.rs        # Core types
│   │   │   ├── lint.rs         # Validation
│   │   │   ├── export.rs       # JSON-ADR support
│   │   │   └── error.rs        # Error types
│   │   └── tests/
│   ├── adrs-cli/         # CLI application
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   └── adrs-mcp/         # MCP server
│       └── src/
├── docs/                 # Documentation (this book)
│   ├── src/
│   └── book.toml
├── tests/                # Workspace-level tests
└── Cargo.toml            # Workspace manifest
```

## Core Concepts

See [Concepts](./concepts.md) for detailed explanations of:

- Library-first architecture
- Dual mode operation
- Template system

## Next Steps

- [Concepts](./concepts.md) - Understand the architecture
- [Library Guide](../lib/README.md) - Using adrs-core
- [CLI Guide](../cli/README.md) - Extending the CLI
- [Testing Guide](../testing/README.md) - Testing strategies
- [Contributing](../contributing.md) - Contribution guidelines
