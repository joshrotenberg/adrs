# 9. Compact file without trailing newline

Date: 2024-05-10

## Status

Accepted

## Context

Some editors and generators emit files without a final newline. This file
intentionally ends without one.

## Decision

We accept ADR files that lack a trailing final newline.

## Consequences

Tools must not corrupt such files when reading or rewriting them.