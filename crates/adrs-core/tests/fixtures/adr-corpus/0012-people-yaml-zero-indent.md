---
number: 12
title: People YAML zero indent consulted
date: 2024-06-02
status: proposed
consulted:
- alice
- bob
---

# 12. People YAML zero indent consulted

## Context

Some authors write YAML lists with zero indentation under the key.

## Decision

Tolerate zero-indent list items in people fields.

## Consequences

- Metadata updates must not orphan list items
- The file must remain parseable after status changes
