# 19. Justfile Argument Attributes

Date: 2026-03-05

## Status

Accepted

## Context

Some recipes benefit from CLI-style flag arguments rather than positional parameters.
For example, `just build --release` is more intuitive than `just build release` for
users familiar with cargo.

Just 1.45.0 introduced the `[arg()]` attribute for flag-style arguments.

## Decision

Use `[arg()]` attributes for flag-style arguments where appropriate, with awareness
of limitations.

### Basic Syntax <sup>1.45.0</sup>

```just
[arg("release", short="r", long="release", value="true")]
build release="false":
    cargo build {{ if release == "true" { "--release" } else { "" } }}
```

Usage:
- `just build` → debug build
- `just build --release` → release build
- `just build -r` → release build

### Attribute Options

| Option | Description | Example |
|--------|-------------|---------|
| First argument | Parameter name (required) | `"release"` |
| `short` | Single-character flag | `short="r"` |
| `long` | Long flag name | `long="release"` |
| `value` | Value when flag present | `value="true"` |

### Full Example

```just
# Build with optional release and target flags
[arg("release", short="r", long="release", value="true")]
[arg("target", long="target")]
build release="false" target="":
    cargo build \
        {{ if release == "true" { "--release" } else { "" } }} \
        {{ if target != "" { "--target " + target } else { "" } }}
```

Usage:
- `just build` → `cargo build`
- `just build -r` → `cargo build --release`
- `just build --target aarch64-apple-darwin` → `cargo build --target aarch64-apple-darwin`
- `just build -r --target x86_64-unknown-linux-gnu` → combined

### Limitations

**Module invocation does not support flags:**

```just
# Root justfile
mod build './crates/justfile'

# This works:
just build::build --release

# This does NOT work:
just build --release  # Error: recipe not found
```

The `[arg()]` attribute only works when invoking the recipe directly, not through
module shorthand.

**Variadic parameters cannot be options:**

```just
# This is NOT allowed:
[arg("files", long="file")]
process +files:  # ERROR: variadic parameters cannot be options
    ...
```

### When to Use

| Use `[arg()]` | Use positional |
|---------------|----------------|
| Boolean flags (`--verbose`, `--release`) | Mode selection (`test unit`) |
| Optional values with defaults | Required values |
| Top-level recipes | Module recipes |
| CLI-like interfaces | Simple recipes |

### Recommendation

For module recipes, prefer positional choice parameters (see [ADR-0018](./0018-justfile-recipe-conventions.md))
since they work with both direct and module invocation:

```just
# Works with: just build release, just build::build release
[default]
build profile="debug":
    @case "{{ profile }}" in \
        debug) cargo build ;; \
        release) cargo build --release ;; \
    esac
```

Reserve `[arg()]` for top-level recipes where the flag interface provides clear UX benefits.

## Consequences

### Positive

- CLI-style interface familiar to users (`--release`, `-v`)
- Self-documenting flags via `just --help`
- Boolean flags don't require explicit values
- Can combine short and long forms

### Negative

- Requires just 1.45.0 or later
- Does not work with module shorthand invocation
- Variadic parameters cannot use `[arg()]`
- More complex syntax than positional parameters

### Neutral

- Flags appear in `just --help` output for the recipe
- Can mix `[arg()]` and positional parameters in same recipe

## References

- [just arg attribute documentation](https://just.systems/man/en/recipe-parameters.html)
- [just 1.45.0 release notes](https://github.com/casey/just/releases/tag/1.45.0)
- [ADR-0014: Justfile Conventions](./0014-justfile-conventions.md) (entrypoint)
- [ADR-0018: Justfile Recipe Conventions](./0018-justfile-recipe-conventions.md)
