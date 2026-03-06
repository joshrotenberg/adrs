# ADR Alignment Matrix

This document compares the current implementation against ADRs 0004-0007.

<!-- toc -->

## Summary

| ADR | Status | Compliance | Notes |
|-----|--------|------------|-------|
| [0004: Library-first Architecture](#adr-0004-library-first-architecture) | Accepted | ✅ High | Architecture matches ADR |
| [0005: Dual-mode Operation](#adr-0005-dual-mode-operation) | Accepted | ⚠️ Partial | `--ng` flag doesn't override template mode |
| [0006: YAML Frontmatter](#adr-0006-yaml-frontmatter) | Accepted | ✅ High | All specified fields supported |
| [0007: Minijinja Templates](#adr-0007-minijinja-templates) | Accepted | ✅ High | Full implementation with extras |

---

## ADR-0004: Library-first Architecture

**ADR Specification:**
- Workspace with two crates: `adrs-core` (library) and `adrs` (CLI)
- CLI provides thin wrapper around adrs-core
- CLI contains only argument parsing and user interaction

**Implementation Status:**

| Requirement | Status | Location |
|-------------|--------|----------|
| Workspace structure | ✅ | `Cargo.toml` |
| `adrs-core` library crate | ✅ | `crates/adrs-core/` |
| `adrs` CLI crate | ✅ | `crates/adrs/` |
| Core types in library | ✅ | `adrs-core/src/types.rs` |
| Parsing in library | ✅ | `adrs-core/src/parse.rs` |
| Config in library | ✅ | `adrs-core/src/config.rs` |
| Templates in library | ✅ | `adrs-core/src/template.rs` |
| Repository ops in library | ✅ | `adrs-core/src/repository.rs` |
| Thin CLI wrapper | ✅ | `adrs/src/commands/*.rs` |

**Additional Implementation:**
- MCP server as feature flag in CLI crate (`mcp` feature)
- Linting module using `mdbook-lint-rulesets`
- Export module for JSON-ADR format

**Potential Gaps:**
- [ ] Audit for `unwrap()`/`expect()` usage in library paths (see B1)
- [ ] Error types could be more granular for library consumers

**Verdict:** ✅ **Aligned** - Implementation matches ADR intent

---

## ADR-0005: Dual-mode Operation

**ADR Specification:**
- Compatible mode (default): `.adr-dir` config, plain markdown, legacy status format
- NextGen mode (opt-in): `adrs.toml` config, YAML frontmatter, enhanced features
- Mode determined by: `--ng` flag, `adrs.toml` presence, or default

**Implementation Status:**

| Requirement | Status | Location |
|-------------|--------|----------|
| `ConfigMode` enum | ✅ | `config.rs:ConfigMode` |
| Compatible mode default | ✅ | `config.rs:DEFAULT_MODE` |
| `.adr-dir` detection | ✅ | `config.rs:discover()` |
| `adrs.toml` detection | ✅ | `config.rs:discover()` |
| `--ng` flag defined | ✅ | `main.rs:Cli` |
| `--ng` used in `init` | ✅ | `commands/init.rs` |
| `--ng` used in `new` for tags | ✅ | `commands/new.rs:85` |
| `--ng` overrides template mode | ⚠️ | Gap - uses config mode |
| Legacy format parsing | ✅ | `parse.rs:parse_legacy()` |
| NextGen format parsing | ✅ | `parse.rs:parse_frontmatter()` |
| Auto-format detection | ✅ | `parse.rs:Parser::parse()` |
| Mode-specific generation | ✅ | `repository.rs:create()` |

**Config Discovery Chain:**
1. Environment (`ADRS_DIR`) ✅
2. Project (`.adr-dir` or `adrs.toml`) ✅
3. Global (`~/.config/adrs/config.toml`) ✅
4. Defaults ✅

**Gap Found:**
Per ADR-0005, mode priority should be: `--ng` flag > config > default. However, `adrs --ng new` on an existing Compatible-mode repo does NOT generate YAML frontmatter. The flag only affects tags validation, not template rendering.

**Potential Fixes:**
- Add `Repository::with_mode()` method to override config mode
- Update CLI to pass override to repository

**Verdict:** ⚠️ **Partial** - Mode handling works but `--ng` flag doesn't fully override

---

## ADR-0006: YAML Frontmatter

**ADR Specification:**
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

**Implementation Status:**

| Field | Status | Type |
|-------|--------|------|
| `number` | ✅ | `u32` |
| `title` | ✅ | `String` |
| `date` | ✅ | `NaiveDate` |
| `status` | ✅ | `AdrStatus` enum |
| `links` | ✅ | `Vec<AdrLink>` |

**Additional Fields Supported:**
| Field | Status | Type | Notes |
|-------|--------|------|-------|
| `decision_makers` | ✅ | `Vec<String>` | Not in ADR spec |
| `consulted` | ✅ | `Vec<String>` | Not in ADR spec |
| `informed` | ✅ | `Vec<String>` | Not in ADR spec |
| `tags` | ✅ | `Vec<String>` | Not in ADR spec |

**Link Types:**
| Kind | Status |
|------|--------|
| `supersedes` | ✅ |
| `superseded_by` | ✅ |
| `amends` | ✅ |
| `amended_by` | ✅ |
| `relates_to` | ✅ |
| Custom | ✅ |

**Potential Gaps:**
- [ ] ADR could be updated to document additional fields now supported
- [ ] Validation of required vs optional fields

**Verdict:** ✅ **Aligned** - Implementation exceeds ADR specification

---

## ADR-0007: Minijinja Templates

**ADR Specification:**
- Use minijinja for template rendering
- Support Nygard and MADR formats
- Support custom user templates
- Conditional output for mode-specific content

**Implementation Status:**

| Requirement | Status | Location |
|-------------|--------|----------|
| minijinja crate | ✅ | `Cargo.toml` |
| Nygard format | ✅ | `template.rs:TEMPLATES` |
| MADR format | ✅ | `template.rs:TEMPLATES` |
| Custom templates | ✅ | `template.rs:render()` |
| Mode conditionals | ✅ | Templates use `{% if mode == "ng" %}` |

**Template Variants:**
| Variant | Description | Status |
|---------|-------------|--------|
| Full | All sections | ✅ |
| Minimal | Essential sections | ✅ |
| Bare | No boilerplate | ✅ |
| BareMinimal | Minimal + no boilerplate | ✅ |

**Template Context Variables:**
| Variable | Type | Status |
|----------|------|--------|
| `number` | int | ✅ |
| `title` | string | ✅ |
| `date` | string | ✅ |
| `status` | string | ✅ |
| `mode` | string | ✅ |
| `author` | string | ✅ |
| `tags` | list | ✅ |

**Custom Filters:**
- `pad(width)` - Left-pad number with zeros ✅

**Potential Gaps:**
- [ ] Template inheritance/includes not documented
- [ ] Custom filter documentation could be expanded

**Verdict:** ✅ **Aligned** - Full implementation with useful extensions

---

## Gap Analysis

### Documented Decisions Needing Updates

1. **ADR-0006 Update** - Document additional frontmatter fields (`decision_makers`, `consulted`, `informed`, `tags`) that are now supported

2. **ADR-0004 Clarification** - Document MCP server as optional feature of CLI crate

### Undocumented Architectural Decisions

| Topic | Description | Suggested ADR |
|-------|-------------|---------------|
| Linting Integration | Uses `mdbook-lint-rulesets` for ADR validation | ADR-0008 |
| JSON-ADR Export | Interoperability format for ADR exchange | ADR-0009 |
| Error Handling Strategy | Thiserror-based error types, Result patterns | ADR-0010 |
| MCP Server | Model Context Protocol integration | ADR-0011 |

---

## Next Steps

1. **Phase B1**: Audit error handling alignment with ADR-0004
2. **Phase B2**: Verify mode handling consistency
3. **Phase B3**: Confirm frontmatter parsing completeness
4. **Phase B4**: Add template rendering tests

See the Phase B alignment plan in `.claude/plans/align-codebase-adrs/phase-b-alignment.md` for details.
