# Architecture Decision Records

This section contains the Architecture Decision Records (ADRs) for the `adrs` project itself.

## What are ADRs?

Architecture Decision Records are short documents that capture important architectural decisions made during a project's development. Each ADR describes a single decision, including:

- **Context**: The situation that requires a decision
- **Decision**: What was decided
- **Consequences**: The implications of the decision

For more background, see [ADR-0001](./0001-record-architecture-decisions.md) and Michael Nygard's [original blog post](http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions).

## ADR Index

| # | Title | Status | Date |
|---|-------|--------|------|
| [0001](./0001-record-architecture-decisions.md) | Record Architecture Decisions | Accepted | 2024-03-04 |
| [0002](./0002-rewrite-it-in-rust.md) | Rewrite it in Rust | Accepted | 2025-01-21 |
| [0003](./0003-use-mdbook-for-documentation.md) | Use mdBook for Documentation | Accepted | 2025-01-21 |
| [0004](./0004-library-first-architecture.md) | Library-first Architecture | Accepted | 2025-01-21 |
| [0005](./0005-dual-mode-compatible-and-nextgen.md) | Dual Mode: Compatible and NextGen | Accepted | 2025-01-21 |
| [0006](./0006-yaml-frontmatter-for-metadata.md) | YAML Frontmatter for Metadata | Accepted | 2025-01-21 |
| [0007](./0007-use-minijinja-for-templates.md) | Use minijinja for Templates | Accepted | 2025-01-21 |
| [0008](./0008-linting-with-mdbook-lint.md) | Linting with mdbook-lint | Accepted | 2026-03-04 |
| [0009](./0009-json-adr-export-format.md) | JSON-ADR Export Format | Accepted | 2026-03-04 |
| [0010](./0010-error-handling-strategy.md) | Error Handling Strategy | Accepted | 2026-03-04 |
| [0011](./0011-mcp-server-integration.md) | MCP Server Integration | Accepted | 2026-03-04 |
| [0012](./0012-mcp-server-library-selection.md) | MCP Server Library Selection | Proposed | 2026-03-04 |
| [0013](./0013-adopt-figment-for-configuration.md) | Adopt Figment for Configuration | Proposed | 2026-03-05 |
| [0014](./0014-justfile-conventions.md) | Justfile Conventions | Accepted | 2026-03-05 |
| [0015](./0015-visual-snapshot-testing.md) | Visual/Snapshot Testing | Accepted | 2026-03-05 |
| [0016](./0016-justfile-module-organization.md) | Justfile Module Organization | Accepted | 2026-03-05 |
| [0017](./0017-justfile-global-settings.md) | Justfile Global Settings | Accepted | 2026-03-05 |
| [0018](./0018-justfile-recipe-conventions.md) | Justfile Recipe Conventions | Accepted | 2026-03-05 |
| [0019](./0019-justfile-argument-attributes.md) | Justfile Argument Attributes | Accepted | 2026-03-05 |

## Status Summary

- **Accepted**: 17
- **Proposed**: 2
- **Superseded**: 0
- **Deprecated**: 0

## How to Read ADRs

ADRs are numbered sequentially. When an ADR supersedes another, the older ADR's status changes to "Superseded" with a reference to the new ADR.

Each ADR follows a consistent structure:

1. **Title**: Number and descriptive name
2. **Date**: When the decision was made
3. **Status**: Current state (Proposed, Accepted, Deprecated, Superseded)
4. **Context**: Background and motivation
5. **Decision**: What was decided
6. **Consequences**: Trade-offs and implications

## Related Documentation

- [Modes](../../users/modes/README.md) - Compatible vs NextGen mode (ADR-0005)
- [Templates](../templates/README.md) - Template formats and variants (ADR-0007)
- [Frontmatter](../../users/frontmatter.md) - YAML metadata format (ADR-0006)
- [Library Usage](../../developers/lib/README.md) - Using adrs-core programmatically (ADR-0004)
