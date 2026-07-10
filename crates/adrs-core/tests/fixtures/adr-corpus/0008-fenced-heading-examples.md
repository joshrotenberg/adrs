---
number: 8
title: Fenced markdown examples in ADR bodies
status: accepted
date: 2024-05-01
---

# 8. Fenced markdown examples in ADR bodies

## Context

Our tooling documentation embeds markdown examples inside ADRs. The fenced
block below contains lines that look like section headings and must never be
treated as real section boundaries by parsers or writers.

```markdown
## Context

Example context inside a fence.

## Consequences

Example consequences inside a fence.
```

Text after the fence still belongs to the Context section.

## Decision

We will keep fenced markdown examples in ADR bodies where they aid
documentation.

## Consequences

- Tools must treat fence content as opaque text
- Section-aware writers must not use fenced heading-lookalikes as boundaries
