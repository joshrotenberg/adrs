---
number: 4
title: Use MADR format for ADRs
status: accepted
date: 2024-02-15
decision-makers:
  - Alice Smith
  - Bob Jones
consulted:
  - Carol White
informed:
  - David Brown
  - Eve Green
---

# 4. Use MADR format for ADRs

## Context

We need to standardize our ADR format. The original Nygard format is simple but lacks structured metadata.

## Decision

We will use MADR 4.0.0 format with YAML frontmatter for new ADRs.

## Consequences

- Better tooling support with structured metadata
- Clear tracking of decision-makers and stakeholders
- Backwards compatible - tools can still parse the markdown
