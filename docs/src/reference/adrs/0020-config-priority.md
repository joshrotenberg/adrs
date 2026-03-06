# 20. Configuration Priority

Date: 2026-03-05

## Status

Proposed

## Context

The adrs tool supports multiple configuration sources:

- Environment variables
- Git config (`[adrs]` section)
- Project config files (`adrs.toml`, `.adr-dir`)
- User global config
- System defaults

We need to define:
1. The precedence order when multiple sources exist
2. How values merge across layers
3. How this maps to future scope concepts

## Decision

### Precedence Order

Configuration sources are layered with higher layers overriding lower layers
on a **per-key basis** (merge semantics, not replacement):

```
Priority  Source                        Scope
────────  ──────                        ─────
1 (high)  ADRS_* env vars               invocation
2         ADRS_CONFIG env var           invocation
3         GIT_DIR/config [adrs]         project (user-local)
4         adrs.toml                     project (version-controlled)
5         .adr-dir                      project (legacy)
6         ADRS_CONFIG_DIR/*             user
7         GIT_CONFIG_GLOBAL [adrs]      user
8         GIT_CONFIG_SYSTEM [adrs]      system
9 (low)   Defaults                      built-in
```

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `ADRS_DIR` | Override `adr_dir` field | `ADRS_DIR=docs/decisions` |
| `ADRS_MODE` | Override `mode` field | `ADRS_MODE=ng` |
| `ADRS_CONFIG` | Point to specific config file | `ADRS_CONFIG=/path/to/adrs.toml` |
| `ADRS_CONFIG_DIR` | Override global config directory | `ADRS_CONFIG_DIR=~/.myconfig` |

The `ADRS_*` pattern allows any config key to be overridden:
- `ADRS_DIR` → `adr_dir`
- `ADRS_MODE` → `mode`
- `ADRS_TEMPLATE_FORMAT` → `templates.format`

### Config Directory Resolution

User global config directory is resolved in order:

```
1. ADRS_CONFIG_DIR env var (if set)
2. XDG_CONFIG_HOME/adrs/ (if XDG_CONFIG_HOME set)
3. ~/.config/adrs/ (default on Unix)
4. %APPDATA%/adrs/ (default on Windows)
```

### Git Config Integration

Git config is read using standard git precedence:

| Source | Location | Git Env Override |
|--------|----------|------------------|
| Local | `.git/config` | `GIT_DIR` |
| Global | `~/.gitconfig` | `GIT_CONFIG_GLOBAL` |
| System | `/etc/gitconfig` | `GIT_CONFIG_SYSTEM` |

The `[adrs]` section in gitconfig uses git-style keys:

```gitconfig
[adrs]
    directory = docs/decisions
    mode = ng
    template-format = madr
```

### Merge Semantics

Values are merged per-key, not replaced wholesale:

```
Layer 1: ADRS_DIR=custom
Layer 2: adrs.toml → { adr_dir: "docs", mode: "ng", templates: { format: "madr" } }
Layer 3: defaults → { adr_dir: "doc/adr", mode: "compatible" }

Result: {
    adr_dir: "custom",        // from Layer 1
    mode: "ng",               // from Layer 2
    templates: {
        format: "madr"        // from Layer 2
    }
}
```

### Future: Scope Mapping

This precedence order aligns with planned scope hierarchy:

```yaml
scopes:
  invocation:           # ADRS_* env, ADRS_CONFIG env
    - highest priority
    - temporary overrides

  project:              # GIT_DIR/config, adrs.toml, .adr-dir
    user-local:         # git local (not version controlled)
    version-controlled: # adrs.toml
    legacy:             # .adr-dir

  user:                 # ADRS_CONFIG_DIR/*, git global
    xdg:                # XDG-aware location
    default:            # ~/.config/adrs/
    git:                # ~/.gitconfig [adrs]

  system:               # git system
    - /etc/gitconfig [adrs]
    - lowest priority
```

### File Format Precedence (Future)

When multiple format support is added:

```
Within same directory: toml > yaml > json
```

If both `adrs.toml` and `adrs.yaml` exist, TOML wins.

## Consequences

### Positive

- Predictable override behavior
- Git-native workflows supported
- Environment overrides for CI/scripting
- Maps cleanly to future scope model
- Per-key merge prevents accidental config loss

### Negative

- Many layers to understand
- Debugging "where did this value come from?" requires verbose mode
- Git config key mapping adds complexity

### Neutral

- Breaking change: new `ConfigSource::GitConfig` variant
- Respecting `GIT_CONFIG_*` env vars may surprise some users

## References

- [ADR-0013: Figment Configuration](./0013-adopt-figment-for-configuration.md)
- [Git Config Documentation](https://git-scm.com/docs/git-config)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
