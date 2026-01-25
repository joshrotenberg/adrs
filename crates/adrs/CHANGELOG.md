# Changelog

All notable changes to this project will be documented in this file.

## [0.5.1] - 2026-01-25

### Documentation

- Refresh README for v0.5.0

### Features

- Add status command to change ADR status
- Add JSON-ADR export support
- Add export --dir and import json commands
- Add --dry-run and --append flags to import command
- Add automatic cross-reference renumbering (Phase 2 of #100)
- Add lint command and integrate mdbook-lint ADR rules ([#98](https://github.com/joshrotenberg/adrs/pull/98))

### Refactoring

- Simplify to just doctor command, remove separate lint

### Testing

- Add integration tests for status command


## [0.5.0] - 2026-01-22

### Bug Fixes

- Add version to adrs-core dependency for cargo publish
- Use workspace dependency for adrs-core

### Documentation

- Rewrite book and improve CLI help for v2

### Features

- V2 rewrite with library-first architecture
- Add MADR 4.0.0 support
- Add template variants (full, minimal, bare)
- Add doctor command for repository health checks
- Add config discovery with directory tree search

### Testing

- Add CLI integration tests
- Add scenario tests for end-to-end workflows

