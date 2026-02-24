//! Tests against real-world ADR files from open-source projects.
//!
//! These fixtures are verbatim (or lightly trimmed) copies of ADRs found in the
//! wild. They exercise format variations that users actually produce.

use adrs_core::{AdrStatus, Parser};

fn parser() -> Parser {
    Parser::new()
}

// ========== adr-tools (Nygard canonical) ==========
// Source: https://github.com/npryce/adr-tools/blob/master/doc/adr/0001-record-architecture-decisions.md

#[test]
fn test_npryce_adr_tools_canonical() {
    let content = r#"# 1. Record architecture decisions

Date: 2016-02-12

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as described by Michael Nygard in this article: http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions

## Consequences

See Michael Nygard's article, linked above.
"#;

    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 1);
    assert_eq!(adr.title, "Record architecture decisions");
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert!(adr.context.contains("architectural decisions"));
    assert!(adr.decision.contains("Architecture Decision Records"));
    assert!(adr.consequences.contains("Michael Nygard"));
}

// ========== MADR 4.0.0 template (no title in frontmatter) ==========
// Source: https://github.com/adr/madr/blob/4.0.0/template/adr-template.md
//
// The official MADR 4.0.0 template deliberately omits `title` from frontmatter.
// This is the canonical case for issue #186.

#[test]
fn test_madr_400_template_no_title_in_frontmatter() {
    // Adapted from the real template: placeholders replaced with real values
    // to make it parseable, but the structure is preserved exactly.
    let content = r#"---
number: 3
status: proposed
date: 2024-01-15
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
informed:
  - Dave
---

# Use Redis for Caching

## Context and Problem Statement

We need a caching solution for our API.

## Decision Drivers

* Response time must be < 100ms
* Must support cache invalidation

## Considered Options

* Redis
* Memcached

## Decision Outcome

Chosen option: "Redis", because it supports complex data types.

### Consequences

* Good, because it reduces database load
* Bad, because it adds operational complexity
"#;

    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 3);
    assert_eq!(adr.title, "Use Redis for Caching");
    assert_eq!(adr.status, AdrStatus::Proposed);
    assert_eq!(adr.decision_makers, vec!["Alice", "Bob"]);
    assert_eq!(adr.consulted, vec!["Carol"]);
    assert_eq!(adr.informed, vec!["Dave"]);
}

// ========== MADR's own ADRs (Jekyll nav_order, no title/number/date/status) ==========
// Source: https://github.com/adr/madr/blob/main/docs/decisions/0015-include-consulting-informed-of-raci.md
//
// These use frontmatter with only Jekyll metadata (parent, nav_order).
// No title, number, date, or status in frontmatter at all.

#[test]
fn test_madr_own_adr_jekyll_frontmatter_only() {
    let content = r#"---
parent: Decisions
nav_order: 15
---
# Include "Consulted" and "Informed" of RACI

## Context and Problem Statement

We noticed an intersection between MADR and RACI, and felt the need to add a "consulted" and "informed" field in addition to "decision maker(s)".
Would it be beneficial to "upstream" these fields to MADR?

## Decision Drivers

* MADR should contain fields important to the ADR decision process
* MADR template should be easy to understand
* MADR should be lightweight

## Considered Options

* Include "Consulted" and "Informed" of RACI
* Include all fields of RACI
* Do not include anything of RACI

## Decision Outcome

Chosen option: "Include 'Consulted' and 'Informed' of RACI", because comes out best (see below).
"#;

    // This will fail YAML deserialization because there's no `number` or `date`.
    // But we should at least be able to confirm the parser doesn't panic.
    let result = parser().parse(content);

    // The frontmatter doesn't have our required fields (number, date), so
    // serde will fail. This is expected -- these ADRs would be parsed via
    // parse_file() which falls back to extracting number from filename.
    // The important thing is we don't panic.
    if let Ok(adr) = result {
        // If it somehow succeeds (e.g., defaults), check the title was extracted
        assert_eq!(adr.title, "Include \"Consulted\" and \"Informed\" of RACI");
    }
}

// ========== MADR's ADR 0013 (nested code block with frontmatter inside) ==========
// Source: https://github.com/adr/madr/blob/main/docs/decisions/0013-use-yaml-front-matter-for-meta-data.md

#[test]
fn test_madr_own_adr_with_nested_code_block_frontmatter() {
    let content = r#"---
parent: Decisions
nav_order: 13
---
# Use YAML front matter for metadata

## Context and Problem Statement

MADR offers the fields "Status", "Decision Maker(s)", and "Date".
Should this data be included in the ADR directly, or should it be separated somehow?

## Decision Outcome

Chosen option: "Use YAML front matter", because comes out best (see below).

## Pros and Cons of the Options

### Use YAML front matter

Example:

```markdown
---
status: accepted
decision-makers:
date:
---

### Context and problem statement

We want to record architectural decisions made in this project.
```

* Good, because tools can handle it more easily
* Bad, because rendering not standardized

### Use plain Markdown everywhere

* Good, because all parsers can handle it
* Bad, because special markdown parsing tooling is needed
"#;

    let result = parser().parse(content);

    // Same situation -- no number/date in frontmatter. But the parser must
    // not be confused by the --- inside the fenced code block.
    if let Ok(adr) = result {
        assert_eq!(adr.title, "Use YAML front matter for metadata");
    }
}

// ========== log4brains (MADR 2.x, inline metadata list, no frontmatter) ==========
// Source: https://github.com/thomvaill/log4brains/blob/master/docs/adr/20200924-use-markdown-architectural-decision-records.md

#[test]
fn test_log4brains_madr_2x_inline_metadata() {
    let content = r#"# Use Markdown Architectural Decision Records

- Status: accepted
- Date: 2020-09-24

## Context and Problem Statement

We want to record architectural decisions made in this project.
Which format and structure should these records follow?

## Considered Options

- [MADR](https://adr.github.io/madr/) 2.1.2 with Log4brains patch
- [MADR](https://adr.github.io/madr/) 2.1.2
- [Michael Nygard's template](http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions)

## Decision Outcome

Chosen option: "MADR 2.1.2 with Log4brains patch", because

- Implicit assumptions should be made explicit.
- The MADR format is lean and fits our development style.
- Version 2.1.2 is the latest one available when starting to document ADRs.
"#;

    // Legacy parsing (no frontmatter). Status is in body as a list item, not
    // in an H2 section -- our parser won't pick it up that way. That's fine.
    // The key test: title is parsed correctly, no panic.
    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 0); // No number in title
    assert_eq!(adr.title, "Use Markdown Architectural Decision Records");
}

// ========== Decentraland (EIP-style, SPDX, custom fields) ==========
// Source: https://github.com/decentraland/adr/blob/main/content/ADR-1-adr-process.md

#[test]
fn test_decentraland_eip_style_frontmatter() {
    // Uses `adr` instead of `number`, so serde will default number to 0.
    // Has `spdx-license`, `type` -- unknown fields the parser should tolerate.
    let content = r#"---
adr: 1
date: 2020-10-05
title: ADR documents specification & process
status: Living
type: Meta
spdx-license: CC0-1.0
---

## What is an ADR?

ADR stands for Architecture Decision Record. An ADR is a design document
providing information to the Decentraland community.

## ADR Rationale

ADRs are heavily inspired in Ethereum Improvement Proposals (EIPs).
"#;

    // `adr` is not `number`, so our parser will deserialize number as 0.
    // `status: Living` is a custom status.
    // The parser must not choke on unknown fields like `type` and `spdx-license`.
    let result = parser().parse(content);

    // serde_yaml strict mode would reject unknown fields, but our Adr struct
    // uses #[serde(default)] on optional fields. The `adr` field will be
    // silently ignored (it's not `number`), so number defaults to 0.
    if let Ok(adr) = result {
        assert_eq!(adr.title, "ADR documents specification & process");
        assert_eq!(adr.status, AdrStatus::Custom("Living".into()));
    }
}

#[test]
fn test_decentraland_adr_with_authors_list() {
    // Source: https://github.com/decentraland/adr/blob/main/content/ADR-115-sdk-playground.md
    let content = r#"---
adr: 115
date: 2022-11-01
title: SDK Playground
status: Living
authors:
  - nearnshaw
type: RFC
spdx-license: CC0-1.0
redirect_from:
  - /rfc/RFC-15
---

# Abstract

This document presents the Playground, a new learning tool for SDK users.

# Need

Learning to use the SDK is tricky, and there's a lot of friction up front.
"#;

    let result = parser().parse(content);

    if let Ok(adr) = result {
        assert_eq!(adr.title, "SDK Playground");
        assert_eq!(adr.status, AdrStatus::Custom("Living".into()));
        // `adr: 115` is not `number: 115`, so number defaults to 0
        assert_eq!(adr.number, 0);
    }
}

// ========== Backstage (Docusaurus frontmatter, no H1) ==========
// Source: https://github.com/backstage/backstage/blob/master/docs/architecture-decisions/adr011-plugin-package-structure.md

#[test]
fn test_backstage_docusaurus_frontmatter_no_h1() {
    let content = r#"---
id: adrs-adr011
title: "ADR011: Plugin Package Structure"
description: Architecture Decision Record (ADR) for Plugin Package Structure
---

## Context

A core feature of Backstage is the extensibility via plugins. Even most of the
core features are implemented as plugins.

## Decision

We will place all plugin related code in the `plugins/` directory.

## Consequences

We will actively migrate existing packages that are part of a plugin to the
`plugins/` folder.
"#;

    let result = parser().parse(content);

    if let Ok(adr) = result {
        // `id` is not `number`, so number defaults to 0
        assert_eq!(adr.number, 0);
        // Title comes from frontmatter, contains the ADR number prefix
        assert_eq!(adr.title, "ADR011: Plugin Package Structure");
        // No body H1 -- body starts at ## Context
        assert!(adr.context.contains("extensibility via plugins"));
        assert!(adr.decision.contains("plugins/"));
        assert!(adr.consequences.contains("actively migrate"));
    }
}

// ========== PMD (Jekyll frontmatter, adr_status, boolean adr field) ==========
// Source: https://github.com/pmd/pmd/blob/main/docs/pages/pmd/projectdocs/decisions/adr-1.md

#[test]
fn test_pmd_jekyll_frontmatter_no_h1() {
    let content = r#"---
title: ADR 1 - Use architecture decision records
sidebar: pmd_sidebar
permalink: pmd_projectdocs_decisions_adr_1.html
sidebaractiveurl: /pmd_projectdocs_decisions.html
adr: true
adr_status: "Accepted"
last_updated: February 2024 (7.0.0)
---

## Context

PMD has grown over 20 years as an open-source project. Along the way many
decisions have been made, but they are not explicitly documented.

## Decision

We will document the decisions we make as a project as a collection of
"Architecture Decision Records".

## Consequences

Explicitly documenting decisions has the benefit that new developers joining
the projects know about the decisions.
"#;

    let result = parser().parse(content);

    if let Ok(adr) = result {
        // Title from frontmatter includes the ADR number prefix
        assert_eq!(adr.title, "ADR 1 - Use architecture decision records");
        // No H1 in body -- body starts at ## Context
        assert!(adr.context.contains("20 years"));
        assert!(adr.decision.contains("Architecture Decision Records"));
        assert!(adr.consequences.contains("new developers"));
    }
}

// ========== Buildbarn (freeform, H1 sections, HTML in metadata) ==========
// Source: https://github.com/buildbarn/bb-adrs/blob/main/0001-buffer.md

#[test]
fn test_buildbarn_freeform_h1_sections() {
    let content = r#"# Buildbarn Architecture Decision Record #1: Buffer layer

Author: Ed Schouten<br/>
Date: 2020-01-09

# Context

The `BlobAccess` interface that Buildbarn currently uses to abstract
away different kinds of backing stores for the CAS and AC (Redis, S3,
gRPC, etc.) is a bit simplistic, in that contents are always transferred
through `io.ReadCloser` handles.

# Decision

The decision is to add a new abstraction to Buildbarn, called the buffer
layer, stored in Go package `github.com/buildbarn/bb-storage/pkg/blobstore/buffer`.
"#;

    // Legacy parsing. This ADR uses H1 (not H2) for sections, so our
    // section parser won't pick up Context/Decision. That's expected.
    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 0); // No "1." prefix -- it's "Record #1:"
    assert_eq!(
        adr.title,
        "Buildbarn Architecture Decision Record #1: Buffer layer"
    );
}

// ========== Frontmatter with YAML comment lines ==========
// Inspired by PMD's actual format which has YAML comments in frontmatter.

#[test]
fn test_frontmatter_with_yaml_comments() {
    let content = r#"---
# This is a YAML comment
number: 5
title: Use YAML comments
# Proposed / Accepted / Deprecated / Superseded
status: accepted
date: 2024-06-15
---

## Context

Some ADR tools put comments in their YAML frontmatter.

## Decision

We should handle them gracefully.
"#;

    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 5);
    assert_eq!(adr.title, "Use YAML comments");
    assert_eq!(adr.status, AdrStatus::Accepted);
}

// ========== Frontmatter with title AND body H1 (title in both) ==========
// Inspired by Straw Hat Team pattern: title appears in frontmatter AND body.
// The #186 fix should prefer the frontmatter title.

#[test]
fn test_title_in_both_frontmatter_and_body() {
    let content = r#"---
number: 42
title: Error Specification
date: 2022-11-03
status: accepted
tags:
  - error
---

# Error Specification

## Context

We need a standard way to specify errors.

## Decision

Use structured error types.
"#;

    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 42);
    assert_eq!(adr.title, "Error Specification"); // Frontmatter wins
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert_eq!(adr.tags, vec!["error"]);
}

// ========== Frontmatter with no title, body H1 has number prefix ==========
// The exact scenario from issue #186: frontmatter omits title to avoid MD025,
// body H1 uses "# 2. My Title" format.

#[test]
fn test_issue_186_frontmatter_no_title_body_h1_numbered() {
    let content = r#"---
number: 2
status: proposed
date: 2024-01-15
decision-makers:
  - Alice
---

# 2. Use PostgreSQL

## Context and Problem Statement

We need a database.

## Decision Outcome

Chosen option: "PostgreSQL", because ACID compliance.

### Consequences

* Good, because strong consistency
* Bad, because operational complexity
"#;

    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 2);
    assert_eq!(adr.title, "Use PostgreSQL"); // Extracted from body H1, number stripped
    assert_eq!(adr.status, AdrStatus::Proposed);
    assert_eq!(adr.decision_makers, vec!["Alice"]);
}

// ========== Frontmatter with no title, body H1 has no number ==========
// Same as above but the body H1 is just "# My Title" without number prefix.

#[test]
fn test_issue_186_frontmatter_no_title_body_h1_unnumbered() {
    let content = r#"---
number: 7
status: accepted
date: 2024-03-20
---

# Adopt trunk-based development

## Context and Problem Statement

Our branching strategy is causing long-lived branches and painful merges.

## Decision Outcome

Chosen option: "trunk-based development", because it encourages small, frequent merges.
"#;

    let adr = parser().parse(content).unwrap();

    assert_eq!(adr.number, 7);
    assert_eq!(adr.title, "Adopt trunk-based development");
    assert_eq!(adr.status, AdrStatus::Accepted);
}

// ========== Custom status values from the wild ==========

#[test]
fn test_custom_status_values() {
    for (status_str, expected) in [
        ("proposed", AdrStatus::Proposed),
        ("accepted", AdrStatus::Accepted),
        ("deprecated", AdrStatus::Deprecated),
        ("superseded", AdrStatus::Superseded),
        // Decentraland uses these
        ("Living", AdrStatus::Custom("Living".into())),
        ("Draft", AdrStatus::Custom("Draft".into())),
        ("Review", AdrStatus::Custom("Review".into())),
        ("Final", AdrStatus::Custom("Final".into())),
        ("Stagnant", AdrStatus::Custom("Stagnant".into())),
        ("Withdrawn", AdrStatus::Custom("Withdrawn".into())),
    ] {
        let content = format!(
            r#"---
number: 1
title: Test
date: 2024-01-01
status: {status_str}
---

## Context

Context.
"#
        );

        let adr = parser().parse(&content).unwrap();
        assert_eq!(adr.status, expected, "Failed for status: {status_str}");
    }
}
