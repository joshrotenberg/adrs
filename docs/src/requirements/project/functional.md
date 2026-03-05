# Functional Requirements

<!-- toc -->

## Core Functionality

### FR-1: adr-tools Compatibility

The tool MUST be a drop-in replacement for adr-tools:

- Read existing `.adr-dir` configuration
- Parse existing ADR files
- Generate compatible output by default

### FR-2: Dual Mode Operation

The tool MUST support two modes:

| Mode | Config | Output |
|------|--------|--------|
| Compatible | `.adr-dir` | Plain markdown |
| NextGen | `adrs.toml` | YAML frontmatter |

### FR-3: Multiple Formats

The tool MUST support:

- **Nygard format**: Classic adr-tools (Context, Decision, Consequences)
- **MADR 4.0.0 format**: Structured format with decision drivers

### FR-4: Template System

The tool MUST provide:

- Built-in templates for each format
- Template variants (full, minimal, bare, bare-minimal)
- Custom template support via configuration

## Repository Operations

### FR-5: Initialization

```sh
adrs init
```

MUST:
- Create configuration file
- Create ADR directory
- Generate initial ADR (0001-record-architecture-decisions.md)

### FR-6: ADR Creation

```sh
adrs new "Title"
```

MUST:
- Auto-assign next number
- Generate file from template
- Open editor for editing
- Support `--no-edit` for scripting

### FR-7: ADR Listing

```sh
adrs list
```

MUST:
- List all ADRs sorted by number
- Show number, title, status
- Support filtering by status

### FR-8: ADR Linking

```sh
adrs link <source> <kind> <target>
```

MUST:
- Create bidirectional links
- Update both ADR files
- Support standard link types (supersedes, amends, relates)

### FR-9: Status Changes

```sh
adrs status <number> <status>
```

MUST:
- Update ADR status
- Handle supersession with `--by`
- Create appropriate links

### FR-10: Search

```sh
adrs search <query>
```

MUST:
- Search title and content
- Return matching ADRs
- Support title-only search

## Import/Export

### FR-11: JSON-ADR Export

```sh
adrs export
```

MUST:
- Export to JSON-ADR format
- Include all ADR metadata
- Support single and bulk export

### FR-12: JSON-ADR Import

```sh
adrs import <file>
```

MUST:
- Import from JSON-ADR format
- Handle conflicts (skip/overwrite)
- Report import results

## Health Checks

### FR-13: Repository Validation

```sh
adrs doctor
```

MUST:
- Detect broken links
- Validate frontmatter
- Check for numbering gaps
- Report issues with severity

## MCP Integration

### FR-14: MCP Server

```sh
adrs mcp serve
```

MUST:
- Implement MCP protocol
- Provide read/write/analysis tools
- Support stdio transport

## See Also

- [Non-Functional Requirements](./non-functional.md)
- [Commands Reference](../../users/commands/README.md)
