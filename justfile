# adrs justfile - Task runner for development workflows
# Run `just --list` to see available recipes

mod schema './schema/justfile'
mod build './crates/justfile'
mod cli './crates/adrs/justfile'
mod lib './crates/adrs-core/justfile'
mod docs './docs/justfile'
mod test './tests/justfile'
mod quality './quality/justfile'

# Remove build artifacts
clean:
    rm -rf "{{ justfile_directory() }}/target/"

# Run all tests (shortcut for test::all)
test-all:
    just test all

# Build debug binary (shortcut for build::debug)
build-debug:
    just build debug

# Build release binary (shortcut for build::release)
build-release:
    just build release

# Install binary locally
install: build-release
    cargo install --path crates/adrs

# Publish crates to crates.io
[confirm]
publish:
    cargo publish --package adrs-core
    cargo publish --package adrs

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
