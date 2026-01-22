# Introduction

`adrs` is a command line tool for creating and managing [Architecture Decision Records](https://adr.github.io) (ADRs).

## What are ADRs?

Architecture Decision Records are short documents that capture important architectural decisions made during a project. Each ADR describes a single decision, including the context, the decision itself, and its consequences.

## Features

- **adr-tools compatible**: Works with existing ADR repositories created by [adr-tools](https://github.com/npryce/adr-tools)
- **Multiple formats**: Supports both Nygard (classic) and [MADR 4.0.0](https://adr.github.io/madr/) formats
- **Template variants**: Full, minimal, and bare templates for each format
- **Cross-platform**: Binaries available for Linux, macOS, and Windows
- **Library support**: Use `adrs-core` as a Rust library in your own tools

## Quick Start

### Installation

Using the shell installer (Linux/macOS):

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/joshrotenberg/adrs/releases/latest/download/adrs-installer.sh | sh
```

Using PowerShell (Windows):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/joshrotenberg/adrs/releases/latest/download/adrs-installer.ps1 | iex"
```

Using Homebrew:

```sh
brew install joshrotenberg/brew/adrs
```

Using Cargo:

```sh
cargo install adrs
```

### Initialize a Repository

```sh
adrs init
```

This creates:
- A `.adr-dir` file pointing to `doc/adr`
- The `doc/adr` directory
- An initial ADR: `0001-record-architecture-decisions.md`

### Create a New ADR

```sh
adrs new "Use PostgreSQL for persistence"
```

This opens your editor with a new ADR from the default template. Save and close to create the record.

### List ADRs

```sh
adrs list
```

### Link ADRs

```sh
adrs link 2 "Amends" 1 "Amended by"
```

## Migration from adr-tools

`adrs` is designed to be a drop-in replacement for adr-tools. Your existing ADR repository will work without changes:

1. Install `adrs`
2. Run commands as usual - `adrs` reads the existing `.adr-dir` file and ADR documents

For new features like MADR format or YAML frontmatter, see the [Configuration](./configuration.md) chapter.

## Next Steps

- [Configuration](./configuration.md) - Learn about configuration options
- [Commands](./commands/README.md) - Detailed command reference
- [Templates](./templates.md) - Customize ADR templates
- [Formats](./formats.md) - Nygard vs MADR format comparison
