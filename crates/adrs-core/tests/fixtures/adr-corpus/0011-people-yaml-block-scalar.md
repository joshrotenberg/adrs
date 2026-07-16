---
number: 11
title: People YAML block scalar consulted
date: 2024-06-01
status: proposed
consulted: >-
  the platform team
---

# 11. People YAML block scalar consulted

## Context

Legal YAML block-scalar people fields must survive metadata updates.

## Decision

Use folded block scalars for multi-word stakeholder names when authors prefer them.

## Consequences

- `adrs status` must not corrupt frontmatter
- The ADR must remain listable after a status change
