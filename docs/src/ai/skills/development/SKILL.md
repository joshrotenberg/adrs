# Development Skills

Skills for AI agents working on the `adrs` codebase.

## Overview

These skills help AI agents contribute to the `adrs` project by:
- Understanding the codebase structure
- Making appropriate changes
- Following project conventions
- Running tests and validation

## Skills

### Understand Architecture

**Trigger**: Questions about project structure or design

**Actions**:
1. Read relevant ADRs in `docs/src/reference/adrs/`
2. Review crate structure in `crates/`
3. Check related documentation

**Key Files**:
- `Cargo.toml` - Workspace structure
- `crates/adrs-core/src/lib.rs` - Core library entry
- `crates/adrs-cli/src/main.rs` - CLI entry

### Add Feature

**Trigger**: Requests to add new functionality

**Actions**:
1. Check if ADR exists for the feature
2. Identify affected crates
3. Implement in `adrs-core` first (library-first)
4. Add CLI/MCP wrappers as needed
5. Write tests
6. Update documentation

**Conventions**:
- All business logic in `adrs-core`
- CLI is a thin wrapper
- Use `thiserror` for error types

### Fix Bug

**Trigger**: Bug reports or unexpected behavior

**Actions**:
1. Reproduce the issue
2. Write failing test
3. Identify root cause
4. Implement fix
5. Verify test passes
6. Check for regression

### Run Tests

**Trigger**: Need to verify changes

**Commands**:
```sh
# All tests
cargo test --all

# Specific crate
cargo test -p adrs-core

# With output
cargo test -- --nocapture
```

### Code Quality

**Trigger**: Before committing changes

**Commands**:
```sh
# Format
cargo fmt --all

# Lint
cargo clippy --all-targets

# Check
cargo check --all
```

### Create ADR

**Trigger**: Significant architectural decision needed

**Actions**:
1. Use `adrs new` to create ADR
2. Document context, alternatives, consequences
3. Link to related ADRs
4. Submit for review
