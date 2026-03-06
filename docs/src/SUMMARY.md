# Summary

[Introduction](./README.md)

---

# Users

- [User Guide](./users/README.md)
  - [Introduction](./users/introduction/README.md)
  - [Installation](./users/installation.md)
  - [Configuration](./users/configuration.md)
- [Modes](./users/modes/README.md)
  - [Compatible Mode](./users/modes/compatible.md)
  - [NextGen Mode](./users/modes/nextgen.md)
- [Frontmatter](./users/frontmatter.md)
- [Commands](./users/commands/README.md)
  - [init](./users/commands/init.md)
  - [new](./users/commands/new.md)
  - [edit](./users/commands/edit.md)
  - [list](./users/commands/list.md)
  - [search](./users/commands/search.md)
  - [status](./users/commands/status.md)
  - [link](./users/commands/link.md)
  - [export](./users/commands/export.md)
  - [import](./users/commands/import.md)
  - [generate](./users/commands/generate.md)
  - [template](./users/commands/template.md)
  - [config](./users/commands/config.md)
  - [doctor](./users/commands/doctor.md)
  - [completions](./users/commands/completions.md)
  - [cheatsheet](./users/commands/cheatsheet.md)

---

# Developers

- [Developer Guide](./developers/README.md)
  - [Introduction](./developers/introduction/README.md)
  - [Concepts](./developers/introduction/concepts.md)
  - [Testing](./developers/testing/README.md)
    - [Test Types](./developers/testing/types/README.md)
      - [Unit Tests](./developers/testing/types/unit-tests.md)
      - [Integration Tests](./developers/testing/types/integration-tests.md)
      - [Property-Based Tests](./developers/testing/types/property-based-tests.md)
    - [Fixtures](./developers/testing/fixtures.md)
    - [Goals](./developers/testing/goals.md)
  - [Library Dev](./developers/lib/README.md)
    - [Modules](./developers/lib/modules/README.md)
      - [Core Types](./developers/lib/modules/types-core.md)
      - [ADR Alignment](./developers/lib/modules/adr-alignment-matrix.md)
    - [Configuration](./developers/lib/configuration.md)
    - [Templates](./developers/lib/templates.md)
    - [Linting](./developers/lib/linting.md)
    - [Import/Export](./developers/lib/import-export.md)
    - [Error Handling](./developers/lib/errors.md)
    - [Requirements](./developers/lib/requirements/README.md)
      - [API](./developers/lib/requirements/api.md)
      - [Types](./developers/lib/requirements/types.md)
      - [Errors](./developers/lib/requirements/errors.md)
      - [Compatibility](./developers/lib/requirements/compatibility.md)
  - [CLI Dev](./developers/cli/README.md)
    - [Examples](./developers/cli/examples/README.md)
      - [Adding a Command](./developers/cli/examples/add-command.md)
    - [Requirements](./developers/cli/requirements/README.md)
      - [Commands](./developers/cli/requirements/commands.md)
      - [Compatibility](./developers/cli/requirements/compatibility.md)
      - [UX](./developers/cli/requirements/ux.md)
  - [MCP Dev](./developers/mcp/README.md)
    - [Tools](./developers/mcp/tools/README.md)
    - [Examples](./developers/mcp/examples/usage.md)
      - [Adding Tools](./developers/mcp/examples/add-tools.md)
    - [Requirements](./developers/mcp/requirements/README.md)
      - [Protocol](./developers/mcp/requirements/protocol.md)
      - [Tools](./developers/mcp/requirements/tools.md)
      - [Safety](./developers/mcp/requirements/safety.md)
      - [Performance](./developers/mcp/requirements/performance.md)
- [Contributing](./developers/contributing.md)
- [Code of Conduct](./developers/code-of-conduct.md)

---

# AI

- [Context Files](./ai/README.md)
  - [Skills](./ai/skills/README.md)
    - [Development](./ai/skills/development/SKILL.md)
    - [Operations](./ai/skills/operations/SKILL.md)
  - [Rules](./ai/rules/README.md)
  - [Hooks](./ai/hooks/README.md)

---

# Requirements

- [Project Requirements](./requirements/project/README.md)
  - [Functional](./requirements/project/functional.md)
  - [Non-Functional](./requirements/project/non-functional.md)

---

# Reference

- [Reference](./reference/README.md)
- [Templates](./reference/templates/README.md)
  - [Nygard Format](./reference/templates/nygard.md)
  - [MADR Format](./reference/templates/madr.md)
  - [Variants](./reference/templates/variants.md)
- [ADRs](./reference/adrs/README.md)
  - [0001: Record Architecture Decisions](./reference/adrs/0001-record-architecture-decisions.md)
  - [0002: Rewrite in Rust](./reference/adrs/0002-rewrite-it-in-rust.md)
  - [0003: Use mdBook](./reference/adrs/0003-use-mdbook-for-documentation.md)
  - [0004: Library-first Architecture](./reference/adrs/0004-library-first-architecture.md)
  - [0005: Dual Mode Operation](./reference/adrs/0005-dual-mode-compatible-and-nextgen.md)
  - [0006: YAML Frontmatter](./reference/adrs/0006-yaml-frontmatter-for-metadata.md)
  - [0007: Use minijinja](./reference/adrs/0007-use-minijinja-for-templates.md)
  - [0008: Linting with mdbook-lint](./reference/adrs/0008-linting-with-mdbook-lint.md)
  - [0009: JSON-ADR Export](./reference/adrs/0009-json-adr-export-format.md)
  - [0010: Error Handling Strategy](./reference/adrs/0010-error-handling-strategy.md)
  - [0011: MCP Server Integration](./reference/adrs/0011-mcp-server-integration.md)
  - [0012: MCP Library Selection](./reference/adrs/0012-mcp-server-library-selection.md)
  - [0013: Figment Configuration](./reference/adrs/0013-adopt-figment-for-configuration.md)
  - [0014: Justfile Conventions](./reference/adrs/0014-justfile-conventions.md)
  - [0015: Visual/Snapshot Testing](./reference/adrs/0015-visual-snapshot-testing.md)
  - [0016: Justfile Module Organization](./reference/adrs/0016-justfile-module-organization.md)
  - [0017: Justfile Global Settings](./reference/adrs/0017-justfile-global-settings.md)
  - [0018: Justfile Recipe Conventions](./reference/adrs/0018-justfile-recipe-conventions.md)
  - [0019: Justfile Argument Attributes](./reference/adrs/0019-justfile-argument-attributes.md)
  - [0020: Configuration Priority](./reference/adrs/0020-config-priority.md)
  - [0021: Testing Strategy](./reference/adrs/0021-testing-strategy.md)
  - [0022: Error Handling Policy](./reference/adrs/0022-error-handling-policy.md)
- [Glossary](./reference/glossary.md)

---

# Resources

- [Resources](./resources/README.md)
- [Roadmap](./roadmap.md)
- [Changelog](./changelog.md)
- [License](./license.md)
