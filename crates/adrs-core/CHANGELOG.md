# Changelog

All notable changes to this project will be documented in this file.

## [0.5.1] - 2026-01-25

### Features

- Add status command to change ADR status
- Add JSON-ADR export support
- Add custom_sections field to JSON-ADR for extensibility
- Add MADR-inspired fields to JSON-ADR spec
- Add export --dir and import json commands
- Add --dry-run and --append flags to import command
- Add automatic cross-reference renumbering (Phase 2 of #100)
- Add lint command and integrate mdbook-lint ADR rules ([#98](https://github.com/joshrotenberg/adrs/pull/98))

### Miscellaneous

- Use published mdbook-lint crates (0.13.7)


## [0.5.0] - 2026-01-22

### Bug Fixes

- Align MADR templates with official adr/madr repository

### Features

- Add doctor command for repository health checks
- Add config discovery with directory tree search

