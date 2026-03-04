# 6. YAML frontmatter for metadata

Date: 2025-01-21

## Status

Accepted

## Context

The original adr-tools format stores metadata (status, links) inline in the markdown content. This approach has several problems:

- Parsing status and links requires regex patterns that can be fragile
- The status section mixes metadata with display formatting
- Links use a specific markdown format that's hard to extend
- Adding new metadata fields requires changing the parsing logic
- Tools that want to read ADR metadata must parse markdown

Many static site generators and documentation tools already use YAML frontmatter as a standard way to store document metadata.

## Decision

In NextGen mode, use YAML frontmatter to store ADR metadata:

```yaml
---
number: 5
title: Use PostgreSQL
date: 2025-01-21
status: accepted
links:
  - target: 3
    kind: supersedes
  - target: 4
    kind: amends
---
```

The frontmatter is:
- Delimited by `---` on its own line
- Valid YAML that can be parsed by standard libraries
- Optional in compatible mode, required in nextgen mode
- Extensible with additional fields as needed

The markdown body still contains the human-readable sections (Context, Decision, Consequences) but without embedded metadata.

## Consequences

- Metadata is cleanly separated from content
- Standard YAML parsers can extract metadata without parsing markdown
- New metadata fields can be added without changing parsing logic
- Tools like static site generators can process ADRs natively
- Files are slightly more verbose with the frontmatter header
- Existing ADRs need migration to add frontmatter (or stay in compatible mode)
