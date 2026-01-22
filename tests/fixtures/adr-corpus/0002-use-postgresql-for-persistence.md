# 2. Use PostgreSQL for persistence

Date: 2024-01-20

## Status

Accepted

Supersedes [1. Record architecture decisions](0001-record-architecture-decisions.md)

## Context

We need a database for storing user data and application state. The team has experience with both MySQL and PostgreSQL. We need ACID compliance and good JSON support.

## Decision

We will use PostgreSQL as our primary database.

## Consequences

- Team needs to ensure PostgreSQL expertise is maintained
- We can leverage advanced features like JSONB and full-text search
- Deployment requires PostgreSQL 14 or later
