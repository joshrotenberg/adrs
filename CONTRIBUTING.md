# Contributing to adrs

Thank you for your interest in contributing to `adrs`! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/0/code_of_conduct/). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- Rust toolchain (stable)
- Git

### Development Setup

```sh
# Clone the repository
git clone https://github.com/joshrotenberg/adrs
cd adrs

# Build
cargo build

# Run tests
cargo test --all

# Run clippy
cargo clippy --all-targets

# Format code
cargo fmt --all
```

### Project Structure

```
adrs/
├── crates/
│   ├── adrs-core/     # Core library (parsing, templates, repository)
│   ├── adrs-cli/      # CLI application
│   └── adrs-mcp/      # MCP server
├── docs/              # Documentation (mdBook)
└── Cargo.toml         # Workspace manifest
```

## How to Contribute

### Reporting Issues

- Check existing issues before creating a new one
- Use the issue templates when available
- Include reproduction steps for bugs
- Provide system information (OS, Rust version)

### Pull Requests

1. **Fork** the repository
2. **Create a branch** for your changes (`git checkout -b feature/my-feature`)
3. **Make your changes** following the guidelines below
4. **Test** your changes (`cargo test --all`)
5. **Lint** your code (`cargo clippy --all-targets`)
6. **Format** your code (`cargo fmt --all`)
7. **Commit** with a descriptive message
8. **Push** to your fork
9. **Open a PR** against the `main` branch

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Fix bug" not "Fixes bug")
- Keep the first line under 72 characters
- Reference issues when applicable (`Fixes #123`)

## Development Guidelines

### Architecture

This project follows a **library-first architecture**:

- All business logic belongs in `adrs-core`
- CLI (`adrs-cli`) is a thin wrapper for argument parsing and output formatting
- MCP server (`adrs-mcp`) is a thin wrapper for JSON-RPC handling

See [ADR-0004](https://joshrotenberg.github.io/adrs/reference/adrs/0004-library-first-architecture.html) for details.

### Code Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write doc comments for public APIs
- Keep functions focused and reasonably sized

### Testing

- Write tests for new functionality
- Maintain existing test coverage
- Use `tempfile` for filesystem tests
- Test both Compatible and NextGen modes where applicable

```sh
# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Run specific crate tests
cargo test -p adrs-core
```

### Documentation

- Update documentation for user-facing changes
- Add doc comments to public APIs
- Update the mdBook if adding features

```sh
# Build documentation
cd docs && mdbook build

# Serve documentation locally
cd docs && mdbook serve
```

## Release Process

Releases are managed by the maintainers using `release-plz`. Contributors don't need to update version numbers or changelogs manually.

## Questions?

- Open a [GitHub Discussion](https://github.com/joshrotenberg/adrs/discussions)
- Check the [documentation](https://joshrotenberg.github.io/adrs/)

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
