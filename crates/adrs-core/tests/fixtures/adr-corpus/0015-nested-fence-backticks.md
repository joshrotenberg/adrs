---
number: 15
title: Nested backtick fences in Decision Outcome
date: 2024-06-05
status: accepted
---

# 15. Nested backtick fences in Decision Outcome

## Context and Problem Statement

ADRs often embed markdown samples that themselves contain fenced blocks.

## Decision Outcome

Chosen option: X.

````md
```
## Consequences

Example consequences inside a nested fence.
```
````

Trailing text after the nested fence.

### Consequences

* Good, because nested samples stay opaque

### Confirmation

Manual review.

## More Information

Optional section that must survive body patches.
