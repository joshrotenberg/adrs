# Introduction to `adrs`

> A CLI for creating & managing [Architecture Decision Records](https://adr.github.io) (ADRs).

## ADRs (Architecture Decision Records)

### The "What"

Architecture Decision Records are short documents that capture important architectural decisions made during a project. Each ADR describes a single decision, including:

- **Context**: The situation that requires a decision
- **Decision**: What was decided
- **Consequences**: The implications of the decision

### The "Why"

ADRs help teams:
- Document why decisions were made
- Onboard new team members
- Review past decisions when circumstances change
- Maintain a decision log for compliance

## Features

- **adr-tools compatible**: Works with existing ADR repositories created by [adr-tools](https://github.com/npryce/adr-tools)
- **Multiple formats**: Supports both Nygard (classic) and [MADR 4.0.0](https://adr.github.io/madr/) formats
- **Template variants**: Full, minimal, and bare templates for each format
- **Cross-platform**: Binaries available for Linux, macOS, and Windows

## Quick Start

### 1. Initialize a Repository

```sh
adrs init
```

This creates:
- A `.adr-dir` file pointing to `doc/adr`
- The `doc/adr` directory
- An initial ADR: `0001-record-architecture-decisions.md`

### 2. Create a New ADR

```sh
adrs new "Use PostgreSQL for persistence"
```

This opens your editor with a new ADR from the default template.

### 3. List ADRs

```sh
adrs list
```

### 4. Link ADRs

```sh
adrs link 2 "Amends" 1
```

## Migration from adr-tools

`adrs` is designed to be a drop-in replacement for adr-tools. Your existing ADR repository will work without changes:

1. Install `adrs`
2. Run commands as usual - `adrs` reads the existing `.adr-dir` file and ADR documents

For new features like MADR format or YAML frontmatter, see [Configuration](../configuration.md).

## Next Steps

- [Installation](../installation.md) - Install `adrs`
- [Configuration](../configuration.md) - Configure your repository
- [Commands](../commands/README.md) - Explore all commands
