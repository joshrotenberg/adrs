# Installation

## Pre-built Binaries

Pre-built binaries are available for Linux, macOS, and Windows.

### Shell Installer (Linux/macOS)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/joshrotenberg/adrs/releases/latest/download/adrs-installer.sh | sh
```

### PowerShell Installer (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/joshrotenberg/adrs/releases/latest/download/adrs-installer.ps1 | iex"
```

### Homebrew (macOS/Linux)

```sh
brew install joshrotenberg/brew/adrs
```

### Manual Download

Download the appropriate binary from the [releases page](https://github.com/joshrotenberg/adrs/releases).

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `adrs-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `adrs-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | x86_64 (Intel) | `adrs-x86_64-apple-darwin.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `adrs-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `adrs-x86_64-pc-windows-msvc.zip` |

## From Source

### Using Cargo

```sh
cargo install adrs
```

### Building from Git

```sh
git clone https://github.com/joshrotenberg/adrs
cd adrs
cargo build --release
```

The binary will be at `target/release/adrs`.

## Docker

A Docker image is available for running `adrs` in containers:

```sh
docker pull ghcr.io/joshrotenberg/adrs:latest
```

Mount your project directory to use it:

```sh
docker run --rm -v $(pwd):/workspace -w /workspace ghcr.io/joshrotenberg/adrs list
```

## Verify Installation

```sh
adrs --version
```

## Shell Completions

Generate shell completions for your shell:

```sh
# Bash
adrs completions bash > ~/.local/share/bash-completion/completions/adrs

# Zsh
adrs completions zsh > ~/.zfunc/_adrs

# Fish
adrs completions fish > ~/.config/fish/completions/adrs.fish

# PowerShell
adrs completions powershell > $PROFILE.CurrentUserAllHosts
```

Note: Shell completions require rebuilding after updating `adrs`.
