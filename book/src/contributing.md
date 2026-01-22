# Contributing

Contributions are welcome! See [issues](https://github.com/joshrotenberg/adrs/issues) for ideas or open a new issue to discuss your proposal.

## Development Setup

```sh
git clone https://github.com/joshrotenberg/adrs
cd adrs
cargo build
```

## Running Tests

```sh
# Library tests
cargo test --lib --all-features

# Integration tests
cargo test --test '*' --all-features

# All tests
cargo test --all-features
```

## Code Quality

Before submitting a PR, ensure:

```sh
# Format code
cargo fmt --all

# Run lints
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features
```

## Guidelines

### Compatibility

The v2 rewrite maintains compatibility with [adr-tools](https://github.com/npryce/adr-tools) repositories. Changes should not break existing ADR directories.

New features that extend beyond adr-tools compatibility should:
- Work in both Compatible and NextGen modes where possible
- Be opt-in via flags or configuration
- Be documented

### Code Style

- Follow Rust 2024 edition idioms
- Use `thiserror` for library errors
- Use `anyhow` for CLI errors
- Add doc comments to public APIs
- Keep functions focused and testable

### Testing

- Add unit tests for new functionality
- Add integration tests for CLI changes
- Test both Compatible and NextGen modes
- Test edge cases (empty files, special characters, etc.)

### Commits

Use [conventional commits](https://www.conventionalcommits.org/):

```text
feat: add new feature
fix: resolve bug
docs: update documentation
test: add tests
refactor: restructure code
chore: maintenance tasks
```

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run tests and lints
5. Commit with conventional commit messages
6. Push and open a PR

## Project Structure

```text
adrs/
├── crates/
│   ├── adrs-core/     # Library crate
│   │   ├── src/
│   │   │   ├── config.rs    # Configuration handling
│   │   │   ├── parse.rs     # ADR parsing
│   │   │   ├── template.rs  # Template engine
│   │   │   ├── repository.rs # Repository operations
│   │   │   ├── doctor.rs    # Health checks
│   │   │   └── types.rs     # Core types
│   │   └── Cargo.toml
│   └── adrs/          # CLI crate
│       ├── src/
│       │   ├── main.rs
│       │   └── commands/    # CLI commands
│       └── Cargo.toml
├── book/              # mdbook documentation
└── Cargo.toml         # Workspace manifest
```

## Documentation

- Update the mdbook in `book/` for user-facing changes
- Add doc comments to public APIs
- Update CHANGELOG.md (handled by release-plz)
