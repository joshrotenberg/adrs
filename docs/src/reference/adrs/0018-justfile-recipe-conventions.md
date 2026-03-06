# 18. Justfile Recipe Conventions

Date: 2026-03-05

## Status

Accepted

## Context

Recipes need consistent patterns for:

- Providing sensible defaults
- Supporting multiple modes/options
- Working across platforms
- Protecting against destructive actions

Just provides attributes and parameters to address these needs.

## Decision

Adopt consistent patterns for recipe definition.

### Default Recipes <sup>1.42.0</sup>

Each module should have a `[default]` recipe providing the most common action:

```just
# Show available recipes
[default]
list:
    @just --list --justfile "{{ justfile() }}"
```

| Module | Default Action |
|--------|---------------|
| root | `just --list` |
| build | `cargo build` (debug) |
| test | unit tests |
| docs | `mdbook serve` |
| quality | all checks |

### Parameterized Recipes

Use choice parameters for recipes with multiple modes. The first positional argument
selects the mode:

```just
# Run tests (unit, doc, integration, all, or smoke)
[default]
test choice="unit":
    @case "{{ choice }}" in \
        unit) cargo nextest run ;; \
        doc) cargo test --doc ;; \
        integration) cargo test --test '*' ;; \
        all) cargo nextest run && cargo test --doc ;; \
        *) echo "Unknown: {{ choice }}. Options: unit, doc, integration, all" && exit 1 ;; \
    esac
```

Usage: `just test`, `just test doc`, `just test all`

### Platform-Specific Recipes <sup>0.5.4</sup>

Use attributes for platform-specific implementations:

```just
[macos]
init:
    brew bundle --file=.config/Brewfile
    pre-commit install

[linux]
init:
    brew bundle --file=.config/Brewfile
    pre-commit install

[windows]
init:
    scoop install pre-commit
    scoop bucket add cargo-bins https://github.com/cargo-bins/scoop-cargo-bins
```

Available platform attributes:

| Attribute | Matches |
|-----------|---------|
| `[linux]` | Linux |
| `[macos]` | macOS |
| `[unix]` | Linux, macOS, BSDs |
| `[windows]` | Windows |

### Confirmation Prompts <sup>1.17.0</sup>

Use `[confirm]` for destructive or irreversible actions:

```just
# Simple confirmation
[confirm]
publish:
    cargo publish --package adrs-core
    cargo publish --package adrs

# Custom confirmation message
[confirm("Install Homebrew? This will run a script from the internet.")]
homebrew:
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### Recipe Documentation

Use comments above recipes for documentation:

```just
# Build workspace (debug, release, or all)
[default]
build profile="debug":
    ...
```

The comment appears in `just --list` output.

### No-CD Attribute <sup>1.9.0</sup>

Prevent directory change for recipes that need to stay in the invocation directory:

```just
[default]
[no-cd]
list:
    @just --list --justfile "{{ justfile() }}"
```

## Consequences

### Positive

- Predictable navigation with `[default]` recipes
- Flexible multi-mode recipes with choice parameters
- Cross-platform support with platform attributes
- Safety for destructive actions with `[confirm]`
- Self-documenting recipes via comments

### Negative

- Platform-specific recipes require maintaining parallel implementations
- Choice parameter errors only caught at runtime
- `[confirm]` can be bypassed with `--yes` flag

### Neutral

- Trade-off between granular recipes vs parameterized recipes
- Some recipes may need both platform-specific and parameterized patterns

## References

- [just recipe attributes](https://just.systems/man/en/recipe-attributes.html)
- [just recipe parameters](https://just.systems/man/en/recipe-parameters.html)
- [ADR-0014: Justfile Conventions](./0014-justfile-conventions.md) (entrypoint)
