# 5. API versioning strategy

Date: 2024-03-01

## Status

Accepted

Amends [3. Use Rust for backend services](0003-use-rust-for-backend.md)

## Context

As our API evolves, we need a strategy for versioning to avoid breaking existing clients.

Options considered:
1. URL path versioning (/v1/users)
2. Header versioning (Accept: application/vnd.api+json;version=1)
3. Query parameter versioning (?version=1)

## Decision

We will use URL path versioning with major version numbers only.

## Consequences

- Clear and visible versioning in URLs
- Easy to route different versions to different services
- May lead to code duplication between versions
- Need to maintain multiple versions simultaneously
