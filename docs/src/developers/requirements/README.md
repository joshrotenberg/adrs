# Project Requirements

This document outlines the high-level requirements for the `adrs` project.

## Functional Requirements

### FR-1: adr-tools Compatibility

The tool MUST be a drop-in replacement for adr-tools:
- Read existing `.adr-dir` configuration
- Parse existing ADR files
- Generate compatible output by default

### FR-2: Dual Mode Operation

The tool MUST support two modes:
- **Compatible mode**: Full adr-tools compatibility
- **NextGen mode**: Enhanced features with YAML frontmatter

### FR-3: Multiple Formats

The tool MUST support:
- Nygard format (classic adr-tools)
- MADR 4.0.0 format

### FR-4: Template System

The tool MUST provide:
- Built-in templates for each format
- Template variants (full, minimal, bare)
- Custom template support

### FR-5: Repository Operations

The tool MUST support:
- Initialize new repository
- Create ADRs with auto-numbering
- Edit existing ADRs
- List and search ADRs
- Link ADRs with bidirectional linking
- Change ADR status
- Import/export in JSON-ADR format

### FR-6: Health Checks

The tool MUST provide repository validation:
- Detect broken links
- Validate frontmatter
- Check for gaps in numbering
- Report issues with severity levels

## Non-Functional Requirements

### NFR-1: Performance

- List 1000 ADRs in < 1 second
- Parse single ADR in < 10ms
- Startup time < 100ms

### NFR-2: Cross-Platform

- Support Linux, macOS, Windows
- Provide pre-built binaries
- Support installation via cargo, homebrew

### NFR-3: Library-First

- Core functionality in `adrs-core` library
- CLI and MCP as thin wrappers
- Library usable independently

### NFR-4: Error Handling

- Clear, actionable error messages
- No panics in library code
- Graceful degradation

## See Also

- [Library Requirements](../lib/requirements/README.md)
- [CLI Requirements](../cli/requirements/README.md)
- [MCP Requirements](../mcp/requirements/README.md)
