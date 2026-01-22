---
number: 7
title: Authentication mechanism
status: accepted
date: 2024-04-01
decision-makers:
  - Security Team
  - Platform Team
consulted:
  - External Security Auditor
links:
  - target: 2
    kind: relates-to
    description: Database choice affects token storage
  - target: 5
    kind: relates-to
    description: API versioning applies to auth endpoints
---

# 7. Authentication mechanism

## Context

We need to implement authentication for our API. Requirements:
- Stateless where possible
- Support for mobile and web clients
- Integration with SSO providers

## Decision

We will use JWT tokens with short expiry and refresh token rotation.

## Consequences

- Stateless authentication reduces database load
- Refresh token rotation improves security
- Need to implement token revocation for logout
- Requires secure storage of refresh tokens on clients
