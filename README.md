# adrs

[![Crates.io Version](https://img.shields.io/crates/v/adrs)](https://crates.io/crates/adrs)
[![crates.io](https://img.shields.io/crates/d/adrs.svg)](https://crates.io/crates/adrs)
[![CI](https://github.com/joshrotenberg/adrs/workflows/CI/badge.svg)](https://github.com/joshrotenberg/adrs/actions?query=workflow%3ACI)
[![dependency status](https://deps.rs/repo/github/joshrotenberg/adrs/status.svg)](https://deps.rs/repo/github/joshrotenberg/adrs)

A command-line tool for creating and managing [Architecture Decision Records](https://adr.github.io) (ADRs).

## Features

- **adr-tools compatible** - works with existing ADR repositories
- **Multiple formats** - supports Nygard (classic) and [MADR 4.0.0](https://adr.github.io/madr/) formats
- **Template variants** - full, minimal, and bare templates
- **Tags support** - categorize ADRs with tags (NextGen mode)
- **Full-text search** - search ADR titles and content
- **Repository health checks** - `doctor` command finds issues
- **Config discovery** - automatically finds ADR directory from subdirectories
- **Import/Export** - JSON-ADR format with federation support
- **MCP server** - AI agent integration via Model Context Protocol
- **Cross-platform** - macOS, Linux, and Windows binaries

## Installation

### Homebrew (macOS/Linux)

```sh
brew install joshrotenberg/brew/adrs
```

### Cargo

```sh
cargo install adrs
```

### Docker

```sh
docker run --rm -v $(pwd):/work ghcr.io/joshrotenberg/adrs init
```

### Binary releases

Download from [GitHub Releases](https://github.com/joshrotenberg/adrs/releases).

## Quick Start

```sh
# Initialize a new ADR repository
adrs init

# Create your first decision
adrs new "Use PostgreSQL for persistence"

# List all ADRs
adrs list

# Check repository health
adrs doctor
```

## Usage

```
adrs [OPTIONS] <COMMAND>

Commands:
  init         Initialize a new ADR repository
  new          Create a new ADR
  edit         Edit an existing ADR
  list         List all ADRs
  search       Search ADRs for matching content
  link         Link two ADRs together
  status       Change an ADR's status
  config       Show configuration
  doctor       Check repository health
  generate     Generate documentation (toc, graph, book)
  export       Export ADRs to different formats
  import       Import ADRs from different formats
  template     Manage ADR templates
  completions  Generate shell completions
  cheatsheet   Show quick reference for common workflows

Options:
      --ng         Enable NextGen mode with YAML frontmatter
  -C, --cwd <DIR>  Run from a different directory
  -h, --help       Print help
  -V, --version    Print version
```

## Examples

### Create ADRs with different formats

```sh
# Classic Nygard format (default)
adrs new "Use REST API"

# MADR 4.0.0 format
adrs new --format madr "Use GraphQL"

# Minimal template
adrs new --variant minimal "Quick decision"

# With tags (NextGen mode)
adrs --ng new --tags security,api "Use JWT for authentication"
```

### Search and filter

```sh
# Full-text search
adrs search postgres

# Filter by status
adrs list --status accepted

# Filter by tag (NextGen mode)
adrs --ng list --tag security
```

### Supersede and link decisions

```sh
# Supersede an existing ADR
adrs new --supersedes 2 "Use MySQL instead"

# Link related ADRs (auto-derives reverse link)
adrs link 3 Amends 1
```

### Generate documentation

```sh
# Table of contents
adrs generate toc > doc/adr/README.md

# Graphviz dependency graph
adrs generate graph | dot -Tsvg > doc/adr/graph.svg

# mdbook
adrs generate book && cd book && mdbook serve
```

### Import/Export

```sh
# Export to JSON-ADR format
adrs export json > decisions.json

# Import from another repository
adrs import decisions.json --renumber
```

## MCP Server (AI Integration)

`adrs` includes an MCP (Model Context Protocol) server for AI agent integration. Build with:

```sh
cargo install adrs --features mcp
```

Add to Claude Desktop config (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "adrs": {
      "command": "adrs",
      "args": ["mcp", "serve"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

The MCP server provides 15 tools for reading, creating, and managing ADRs.

## Library

`adrs` is built on the `adrs-core` library, which can be used independently:

```toml
[dependencies]
adrs-core = "0.6"
```

```rust
use adrs_core::Repository;

let repo = Repository::open(".")?;
for adr in repo.list()? {
    println!("{}: {}", adr.number, adr.title);
}
```

See [library documentation](https://docs.rs/adrs-core) for more details.

## Documentation

Full documentation: [joshrotenberg.github.io/adrs-book](https://joshrotenberg.github.io/adrs-book/)

## Contributing

Contributions welcome! See [issues](https://github.com/joshrotenberg/adrs/issues) or open a new one.

## License

MIT or Apache-2.0
