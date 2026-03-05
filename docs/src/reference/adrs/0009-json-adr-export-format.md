# 9. JSON-ADR export format

Date: 2025-03-04

## Status

Proposed

## Context

ADRs stored as markdown files are human-readable but harder to process programmatically. Use cases requiring structured data:

- Integration with external tools (dashboards, wikis)
- Migration between ADR tools
- Bulk operations and analysis
- API responses (MCP server)

Options considered:

1. **Custom JSON schema**: Define our own structure
2. **JSON-ADR format**: Emerging standard for ADR interchange
3. **YAML export**: Alternative structured format

## Decision

Implement JSON-ADR format support for import/export operations because:

- Standard format improves interoperability
- JSON is widely supported across languages and tools
- Enables round-trip capability (export → edit → import)
- MCP server can return structured ADR data

The `export` module provides:
- `export_adr()`: Convert single ADR to JSON
- `export_repository()`: Bulk export all ADRs
- `import_to_directory()`: Import ADRs from JSON

Export preserves all ADR metadata:
- Number, title, date, status
- Links with kind and description
- Tags, decision makers, consulted, informed
- Body sections (context, decision, consequences)

## Consequences

- ADRs can be exchanged with other tools
- Enables programmatic ADR management
- JSON schema must stay in sync with ADR types
- Slightly larger file sizes than markdown
