# 13. Adopt figment for configuration

Date: 2026-03-05

## Status

Proposed

## Context

The adrs configuration system currently uses manual loading and precedence logic. This approach has served well but has limitations:

1. **Precedence logic is manual** - Each new config source requires updating the discovery algorithm
2. **Error messages are basic** - No indication of which layer provided which value
3. **Testing is complex** - Mocking requires setup at multiple levels
4. **Future expansion is costly** - Adding gitconfig support (D4.2) would further complicate the logic

The Rust ecosystem offers several configuration libraries:

| Library     | Layered | Typed | Env vars | Custom providers |
| ----------- | ------- | ----- | -------- | ---------------- |
| **figment** | ✅      | ✅    | ✅       | ✅               |
| config-rs   | ✅      | ✅    | ✅       | Limited          |
| envy        | ❌      | ✅    | ✅       | ❌               |
| confy       | ❌      | ✅    | ❌       | ❌               |

## Decision

Adopt [figment](https://docs.rs/figment) for configuration management because:

1. **Native layered merging** - Configuration sources stack naturally with clear precedence
2. **Custom providers** - Can implement providers for `.adr-dir` files and gitconfig
3. **Excellent error messages** - Shows which provider contributed each value
4. **Type-safe extraction** - Direct deserialization into `Config` struct
5. **Profile support** - Built-in support for dev/prod environments (future use)
6. **Maintained by Rocket author** - Sergio Benitez, stable and well-documented

### Requirements

#### Precedence Order (highest to lowest)

1. `ADRS_CONFIG` environment variable (explicit config path)
2. `ADR_DIRECTORY` environment variable (directory override)
3. `adrs.toml` in project root
4. `.adr-dir` in project root (legacy format)
5. `~/.config/adrs/config.toml` (global config)
6. Built-in defaults

#### Environment Variable Handling

- `ADRS_CONFIG`: Path to explicit config file (overrides all discovery)
- `ADR_DIRECTORY`: Override for `adr_dir` field only
- Both prefixes must be supported for backward compatibility

#### Project Root Discovery

- Search upward from current directory
- Stop at `.git` directory boundary
- Recognize: `adrs.toml`, `.adr-dir`, or `doc/adr` directory

#### Configuration Validation

- `adr_dir` must not be empty
- `mode` must be valid enum value (`compatible` or `ng`)
- Invalid config produces clear error with source location

#### CLI Enhancements

- `adrs config show` - Display resolved configuration
- `adrs config show --verbose` - Show all layers and sources
- `adrs config migrate --to <format>` - Convert between formats

### Backward Compatibility

- `Config` struct unchanged
- `ConfigMode` enum unchanged
- `ConfigSource` enum unchanged (extended in D4.2)
- `DiscoveredConfig` struct unchanged
- `discover()` function signature unchanged

All existing code using these types continues to work.

## Consequences

### Positive

- Cleaner configuration code (~40% reduction in config.rs)
- Better error messages with source attribution
- Easy to add new config sources (gitconfig in D4.2)
- Simpler testing with mock providers
- Foundation for advanced features (profiles, validation)

### Negative

- New dependency (figment ~50KB)
- Team needs to learn figment API
- Slightly different error types (mapped to existing Error)

### Neutral

- Same precedence rules, different implementation
- Same config file formats supported
