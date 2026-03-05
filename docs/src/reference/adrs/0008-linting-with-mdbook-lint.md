# 8. Linting with mdbook-lint-rulesets

Date: 2025-03-04

## Status

Proposed

## Context

ADRs follow conventions that can be validated programmatically:

- Required sections (Context, Decision, Consequences)
- Status field with valid values
- Proper frontmatter format in NextGen mode
- Valid links between ADRs
- Consistent date formats

Manual review is error-prone and time-consuming. An automated linting system would:

- Catch formatting issues early
- Enforce consistency across ADRs
- Integrate with CI pipelines
- Provide actionable feedback

Options considered:

1. **Custom linting rules**: Build validation from scratch
2. **mdbook-lint-rulesets**: Leverage existing ADR-specific rules (ADR001-ADR017)
3. **Generic markdown linters**: markdownlint, remark-lint

## Decision

Use `mdbook-lint-rulesets` for ADR validation because:

- Provides ADR-specific rules (ADR001 through ADR017)
- Maintained and well-documented
- Integrates with the existing mdBook documentation toolchain
- Severity levels (Info, Warning, Error) allow flexible enforcement
- Rules can be enabled/disabled via configuration

The `lint` module exposes:
- `lint_adr()`: Validate a single ADR
- `lint_all()`: Validate all ADRs in repository
- `check_repository()`: Full repository health check

## Consequences

- Consistent ADR quality enforced automatically
- CI can fail on linting errors
- `adrs doctor` command provides user-facing lint results
- Dependency on `mdbook-lint-core` and `mdbook-lint-rulesets` crates
- Rule updates require dependency version bumps
