---
number: 16
title: Tilde fence wrapping backtick sample
date: 2024-06-06
status: accepted
---

# 16. Tilde fence wrapping backtick sample

## Context and Problem Statement

CommonMark allows tilde fences (`~~~`) as well as backticks. Docs often wrap a
backtick sample in an outer tilde fence so the sample can show ` ``` ` lines.

## Decision Outcome

Chosen option: Y.

~~~md
```markdown
## Consequences

Example consequences inside a tilde-wrapped fence.
```
~~~

Trailing text after the tilde fence.

### Consequences

* Good, because mixed fence markers stay opaque

### Confirmation

Manual review.
