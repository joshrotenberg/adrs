# 4. Library-first architecture

Date: 2025-01-21

## Status

Accepted

## Context

The original adrs implementation was a single crate that mixed CLI concerns with core logic. This made it difficult to:

- Test the core functionality in isolation
- Reuse the ADR logic in other tools or libraries
- Maintain clean separation of concerns
- Extend the tool with new features without touching CLI code

Other Rust CLI tools have successfully used a library-first approach where the core logic lives in a separate crate from the CLI.

## Decision

Restructure adrs as a Cargo workspace with two crates:

- `adrs-core`: The library crate containing all ADR types, parsing, configuration, templates, and repository operations
- `adrs`: The CLI crate that provides a thin wrapper around adrs-core

The CLI should contain only argument parsing and user interaction, delegating all business logic to the library.

## Consequences

- The core library can be used by other tools (IDE plugins, web services, etc.)
- Testing is simpler with clear boundaries between components
- The CLI remains thin and focused on user experience
- Breaking changes in the library are clearly visible through semver
- Slightly more complex project structure with workspace configuration
