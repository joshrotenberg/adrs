# Developer Introduction

This guide provides an overview of the `adrs` project architecture for developers.

## Project Structure

```
adrs/
├── crates/
│   ├── adrs-core/        # Core library
│   │   ├── src/
│   │   │   ├── adr.rs          # ADR type definitions
│   │   │   ├── config.rs       # Configuration handling
│   │   │   ├── repository.rs   # Repository operations
│   │   │   ├── template.rs     # Template engine
│   │   │   └── ...
│   │   └── tests/
│   ├── adrs-cli/         # CLI application
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   └── adrs-mcp/         # MCP server
│       └── src/
├── docs/                 # Documentation (this book)
└── Cargo.toml            # Workspace manifest
```

## Key Design Decisions

The project's architecture is documented in ADRs:

- [ADR-0004: Library-first Architecture](../../reference/adrs/0004-library-first-architecture.md)
- [ADR-0005: Dual Mode Operation](../../reference/adrs/0005-dual-mode-compatible-and-nextgen.md)
- [ADR-0006: YAML Frontmatter](../../reference/adrs/0006-yaml-frontmatter-for-metadata.md)
- [ADR-0007: minijinja Templates](../../reference/adrs/0007-use-minijinja-for-templates.md)

## Core Concepts

### Library-First

All business logic lives in `adrs-core`. The CLI and MCP server are thin wrappers that handle:
- CLI: Argument parsing, user interaction, output formatting
- MCP: JSON-RPC handling, tool registration

### Dual Mode

The tool operates in two modes:
- **Compatible**: Full adr-tools compatibility
- **NextGen**: Enhanced features with YAML frontmatter

### Template System

Templates use minijinja (Jinja2 syntax) with:
- Built-in formats: Nygard, MADR
- Variants: full, minimal, bare, bare-minimal
- Custom template support

## Development Setup

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
cd docs && mdbook build
```

## Next Steps

- [Library Guide](../lib/README.md) - Using adrs-core
- [Testing Guide](../testing/README.md) - Testing strategies
- [Contributing](../contributing.md) - Contribution guidelines
