# 22. Error Handling Policy

Date: 2026-03-05

## Status

Proposed

## Context

ADR-0010 defines the error handling *mechanism* (thiserror, Result types). This ADR
defines error handling *policy*: how the application should behave when encountering
errors, particularly during configuration loading.

Key considerations:
- Users may commit invalid configs (intentionally or accidentally)
- External systems (git, filesystem) may be unavailable or restricted
- Forward compatibility requires tolerance of unknown fields
- Developer experience requires actionable error messages

## Decision

### Error Categories

| Category | Description | Examples |
|----------|-------------|----------|
| **Environmental** | External system constraints | Permission denied, disk full, network unavailable |
| **Internal** | Bugs or unexpected states | Parse error that passed linting, assertion failures |
| **Configuration** | Invalid user-provided settings | Lint errors, invalid values, malformed TOML/gitconfig |
| **Runtime** | Valid config but invalid state | ADR directory doesn't exist, referenced ADR not found |

### Error Levels

All errors are logged. Level determines user visibility and operation continuity.

| Level | Behavior | Use Case |
|-------|----------|----------|
| **Fatal** | Panic, operation fails immediately | Unrecoverable internal errors |
| **Error** | Operation fails, can be skipped/recovered | Runtime errors, missing required resources |
| **Warn** | Continue with degraded behavior | Invalid config values, permission issues |
| **Info** | Continue normally, notable event | Config layer skipped, fallback used |
| **Debug** | Development diagnostics | Config resolution steps, parse details |
| **Trace** | Fine-grained diagnostics | Individual key lookups, type coercions |

### Graceful Degradation Strategy

```
On error → Warn → Log → Fall back to defaults → Log fallback used
```

1. Log the original error at appropriate level
2. Apply fallback behavior (skip layer, use default value)
3. Log what fallback was applied
4. Continue operation

### Policy Principles

1. **All configs are advisory** - Errors warn, not fail. Users are ALLOWED to commit
   bad configs. The tool should not block workflows due to config issues.

2. **Unknown keys are ignored** - Enables forward/backward compatibility. A newer
   config can be used with older versions without errors.

3. **Type mismatches degrade gracefully** - Use default for that key, warn user.
   `mode = 123` becomes `mode = "compatible"` with a warning.

4. **Missing files are silent** - Absence of optional config is normal. No warning
   for missing `~/.config/adrs/config.toml` or `/etc/gitconfig`.

5. **Permission errors are silent** - Common on shared systems. Skip inaccessible
   config sources without warning.

### Configuration Loading Behavior

| Scenario | Level | Behavior |
|----------|-------|----------|
| TOML syntax error | Warn | Skip file, use remaining layers |
| Gitconfig parse error | Warn | Skip `[adrs]` section |
| Unknown key in config | Debug | Ignore key |
| Wrong type for known key | Warn | Skip key, use default |
| Empty value for key | Debug | Treat as unset, skip key |
| Invalid enum value | Warn | Use default, report valid options |
| File not found | Silent | Expected for optional configs |
| Permission denied | Silent | Skip source |
| All layers fail | Info | Use built-in defaults |

### Key Mapping Policy (Gitconfig)

| Aspect | Policy | Rationale |
|--------|--------|-----------|
| Unknown keys | Ignore + Debug log | Forward compatibility; configurable to warn/error |
| Case sensitivity | Case-insensitive | Git convention |
| Empty values | Treat as unset | Skip key, no warning |
| Type mismatch | Warn + skip key | Graceful degradation; use default for that key |

**Note:** "Type mismatch" means a value that parses but is the wrong type for the field
(e.g., `mode = 123` instead of `mode = "ng"`). The key is skipped with a warning,
and the default value is used. This aligns with Policy Principle #3.

### Verbose Mode

`--verbose` reveals the full configuration resolution:

```
Config Resolution:
  ✓ Layer 1: ADRS_DIR=custom (env)
  ✓ Layer 2: adrs.toml (project)
  ⚠ Layer 3: .git/config [adrs] - parse error on line 5, skipped
  - Layer 4: ~/.gitconfig [adrs] - not found
  ✓ Layer 5: defaults

Result:
  adr_dir: "custom" (from: env ADRS_DIR)
  mode: "ng" (from: adrs.toml)
  templates.format: "madr" (from: defaults)
```

### Doctor Integration

`adrs doctor` reports configuration health:

```
Configuration:
  ✓ adrs.toml valid
  ⚠ .git/config [adrs] has parse error (line 5: invalid key)
  ✓ ~/.config/adrs/config.toml valid
  ℹ Using 2 of 3 config sources
```

## Consequences

### Positive

- Tool never blocks workflows due to config issues
- Forward/backward compatible configs
- Clear diagnostic path via `--verbose` and `doctor`
- Predictable fallback behavior

### Negative

- Silent failures may hide user mistakes
- Users must use `--verbose` or `doctor` to diagnose issues
- More complex implementation (graceful degradation paths)

### Neutral

- Trade-off between strict validation and workflow continuity
- Debug logging may be verbose in development

## References

- [ADR-0010: Error Handling Strategy](./0010-error-handling-strategy.md) - Mechanism (thiserror)
- [ADR-0020: Configuration Priority](./0020-config-priority.md) - Precedence order
- [Rust tracing crate](https://docs.rs/tracing/) - Logging levels

