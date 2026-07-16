---
number: 13
title: People YAML comment between key and items
date: 2024-06-03
status: proposed
consulted:
  # people
  - alice
---

# 13. People YAML comment between key and items

## Context

YAML allows comments between a key and its list items.

## Decision

Preserve comment-separated people-field lists through metadata writes.

## Consequences

- Values must not duplicate on every update
- Frontmatter must stay valid YAML
