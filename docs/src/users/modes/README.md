# Modes

`adrs` operates in one of two modes that determine how ADRs are stored and formatted:

- **Compatible Mode** (default): Full compatibility with adr-tools
- **NextGen Mode**: Enhanced features with YAML frontmatter

## Quick Comparison

| Aspect | Compatible | NextGen |
|--------|------------|---------|
| Config file | `.adr-dir` | `adrs.toml` |
| Metadata storage | Inline in markdown | YAML frontmatter |
| adr-tools compatible | Yes | No |
| MADR fields | Limited | Full support |

## Mode Selection

The mode is determined by (in order of precedence):

1. **Command-line flag**: `--ng` forces NextGen mode
2. **Configuration file**: `adrs.toml` with `mode = "nextgen"`
3. **Default**: Compatible mode

```sh
# Use NextGen mode for one command
adrs --ng new "My Decision"

# Initialize a new repo in NextGen mode
adrs init --ng
```

## Interoperability

- `adrs` can **read both formats** regardless of mode
- The parser auto-detects the format
- Writing/updating uses the configured mode
- Mixed repositories work for reading but not recommended

## Detailed Documentation

- [Compatible Mode](./compatible.md) - Full adr-tools compatibility
- [NextGen Mode](./nextgen.md) - Enhanced features and YAML frontmatter

## Related

- [Frontmatter](../frontmatter.md) - YAML metadata reference (NextGen)
- [Templates](../../reference/templates/README.md) - Format and variant options

> **Related:** [ADR-0005: Dual Mode Operation](../../reference/adrs/0005-dual-mode-compatible-and-nextgen.md)
