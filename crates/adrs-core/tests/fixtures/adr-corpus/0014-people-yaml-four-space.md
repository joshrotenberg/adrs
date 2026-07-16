---
number: 14
title: People YAML four space indented list
date: 2024-06-04
status: proposed
consulted:
    - alice
    - bob
---

# 14. People YAML four space indented list

## Context

Some serializers indent YAML list items with four spaces.

## Decision

Tolerate four-space indentation for people-field lists.

## Consequences

- Metadata updates must not absorb or corrupt indented items
- The ADR must remain parseable after status changes
