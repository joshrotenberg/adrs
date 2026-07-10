# 10. Non-canonical status prose

Date: 2024-05-20

## Status

Approved by the architecture board on 2024-05-18.

## Context

Some teams record status as free-form prose rather than a canonical keyword.
Files like this are produced by other ADR tools and by hand.

## Decision

We tolerate non-canonical status text when reading ADRs written by other
tools.

## Consequences

Parsers must not silently rewrite or discard unrecognized status prose.
