# 17. Justfile Global Settings

Date: 2026-03-05

## Status

Accepted

## Context

Recipes often need consistent shell behavior for error handling and variable expansion.
Without global settings, each recipe requires its own shebang and error handling setup,
leading to:

- Duplicated boilerplate across recipes
- Inconsistent error handling (some recipes fail silently)
- Verbose multi-line recipes for simple commands

Just provides `set` directives for project-wide configuration.

## Decision

Use `set` directives for consistent shell and environment configuration.

### Shell Configuration <sup>0.5.0</sup>

Configure bash with strict error handling:

```just
set shell := ["/usr/bin/env", "bash", "-eu", "-o", "pipefail", "-c"]
```

| Flag | Purpose |
|------|---------|
| `-e` | Exit on first error |
| `-u` | Error on undefined variables |
| `-o pipefail` | Pipeline fails if any command fails |
| `-c` | **Must be last** - just appends command as argument |

### Temp Directory <sup>1.10.0</sup>

Configure temp directory for the `\`tempdir\`` function:

```just
set tempdir := "/tmp"
```

Usage in recipes:
```just
test:
    #!/usr/bin/env bash
    WORK_DIR=$(mktemp -d)
    # Or use just's tempdir function in recipe body
```

### Working Directory <sup>1.9.0</sup>

Set working directory for module recipes:

```just
# In crates/justfile
set working-directory := ".."
```

This is essential for modules in subdirectories that need to run cargo commands
from the workspace root.

### All Settings

| Setting | Version | Default | Purpose |
|---------|---------|---------|---------|
| `shell` | <sup>0.5.0</sup> | `["sh", "-cu"]` | Shell for recipe execution |
| `tempdir` | <sup>1.10.0</sup> | system temp | Directory for temp files |
| `working-directory` | <sup>1.9.0</sup> | justfile location | Recipe working directory |
| `dotenv-load` | <sup>0.9.4</sup> | `false` | Load `.env` file |
| `positional-arguments` | <sup>0.9.6</sup> | `false` | Pass args as `$1`, `$2`, etc. |

## Consequences

### Positive

- Consistent error handling across all recipes
- Reduced boilerplate (no per-recipe shebangs for simple commands)
- Undefined variable errors caught immediately
- Pipeline failures don't get swallowed
- Portable shell invocation with `/usr/bin/env`

### Negative

- Strict mode may break recipes that expect lenient behavior
- Contributors must understand bash strict mode semantics
- `set -u` requires careful handling of optional variables

### Neutral

- Settings are inherited by modules (can be overridden per-module)
- Some recipes may still need explicit shebangs for complex scripts

## References

- [just settings documentation](https://just.systems/man/en/settings.html)
- [Bash strict mode](http://redsymbol.net/articles/unofficial-bash-strict-mode/)
- [ADR-0014: Justfile Conventions](./0014-justfile-conventions.md) (entrypoint)
