# adrs justfile - Task runner for development workflows
# See ADR-0014 for justfile conventions

# Global shell settings (ADR-0014)
set shell := ["/usr/bin/env", "bash", "-eu", "-o", "pipefail", "-c"]
set tempdir := "/tmp"

# JSON Schema generation and validation
mod schema './schema/justfile'

# Workspace build recipes (debug, release)
mod build './crates/justfile'

# CLI binary recipes (run, demo, smoke tests)
mod cli './crates/adrs/justfile'

# Core library recipes (tests, benchmarks)
mod lib './crates/adrs-core/justfile'

# Documentation recipes (mdbook, rustdoc)
mod docs './docs/justfile'

# Test recipes (unit, integration, visual)
mod test './tests/justfile'

# Code quality recipes (lint, security, coverage)
mod quality './quality/justfile'

# ============================================================================
# Default and Common Recipes
# ============================================================================

# Show available recipes
[default]
[no-cd]
list:
    @just --list --justfile "{{ justfile() }}"

# Remove all build artifacts (workspace-wide)
clean:
    rm -rf "{{ justfile_directory() }}/target/"

# ============================================================================
# Installation
# ============================================================================

# Install binary locally (requires release build)
install:
    just build release
    cargo install --path crates/adrs

# Publish crates to crates.io
[confirm]
publish:
    cargo publish --package adrs-core
    cargo publish --package adrs

# ============================================================================
# Development Environment Setup
# ============================================================================

# Install Homebrew package manager
[macos]
[confirm("Install Homebrew? This will run a script from the internet.")]
homebrew:
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Homebrew package manager
[linux]
[confirm("Install Homebrew? This will run a script from the internet.")]
homebrew:
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Initialize development environment
[macos]
init:
    brew bundle --file=.config/Brewfile
    pre-commit install
    pre-commit install --hook-type pre-push
    cargo update

# Initialize development environment
[linux]
init:
    brew bundle --file=.config/Brewfile
    pre-commit install
    pre-commit install --hook-type pre-push

# Initialize development environment
[windows]
init:
    scoop install pre-commit
    scoop bucket add cargo-bins https://github.com/cargo-bins/scoop-cargo-bins
    scoop install cargo-nextest cargo-deny cargo-machete
    pre-commit install
    pre-commit install --hook-type pre-push
