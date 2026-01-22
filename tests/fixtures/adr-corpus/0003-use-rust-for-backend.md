# 3. Use Rust for backend services

Date: 2024-02-01

## Status

Proposed

## Context

We need to choose a language for our backend services. Key requirements:
- High performance for data processing
- Memory safety
- Good async support
- Strong type system

The team has varying levels of experience with Go, Rust, and Java.

## Decision

We will use Rust for backend services.

## Consequences

- Steeper learning curve for some team members
- Excellent performance characteristics
- Memory safety without garbage collection
- Strong ecosystem for web services (axum, tokio)
- Longer initial development time, but fewer runtime bugs
