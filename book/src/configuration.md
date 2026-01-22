# Configuration

`adrs` supports multiple configuration methods with the following priority (highest to lowest):

1. Command-line flags
2. Environment variables
3. Project configuration file (`adrs.toml` or `.adr-dir`)
4. Global configuration file (`~/.config/adrs/config.toml`)
5. Default values

## Project Configuration

### Legacy Format (.adr-dir)

For compatibility with adr-tools, `adrs` reads the `.adr-dir` file:

```
doc/adr
```

This single-line file specifies the directory where ADRs are stored.

### TOML Format (adrs.toml)

For more options, use `adrs.toml` in your project root:

```toml
# ADR storage directory (relative to project root)
adr_dir = "doc/adr"

# Operation mode: "compatible" or "nextgen"
# - compatible: adr-tools compatible output (default)
# - nextgen: YAML frontmatter with enhanced features
mode = "compatible"

# Template configuration
[templates]
# Default format: "nygard" or "madr"
format = "nygard"

# Default variant: "full", "minimal", or "bare"
variant = "full"

# Path to custom template file (optional)
# custom = "templates/custom.md"
```

## Global Configuration

Create `~/.config/adrs/config.toml` for user-wide defaults:

```toml
# Default mode for new repositories
mode = "nextgen"

[templates]
format = "madr"
variant = "minimal"
```

Project configuration overrides global configuration.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ADR_DIRECTORY` | Override the ADR directory path |
| `ADRS_CONFIG` | Path to a specific configuration file |
| `EDITOR` | Editor to use for `adrs new` and `adrs edit` |

Example:

```sh
export ADR_DIRECTORY="decisions"
adrs new "Use Redis for caching"
```

## Configuration Discovery

When you run an `adrs` command, it searches for configuration by:

1. Looking in the current directory for `adrs.toml` or `.adr-dir`
2. Searching parent directories up to the git root (or filesystem root)
3. Checking the global config at `~/.config/adrs/config.toml`
4. Using defaults if nothing is found

This means you can run `adrs` commands from any subdirectory of your project.

## Modes

### Compatible Mode (default)

Produces output identical to adr-tools:

- Status in the document body
- Links as markdown text
- No YAML frontmatter

### NextGen Mode

Enables enhanced features:

- YAML frontmatter for metadata
- Structured status and links
- MADR 4.0.0 fields (decision-makers, consulted, informed)

Enable with `--ng` flag or in configuration:

```toml
mode = "nextgen"
```

## Show Current Configuration

```sh
adrs config
```

Output:

```
Project root: /path/to/project
Config source: adrs.toml
ADR directory: doc/adr
Full path: /path/to/project/doc/adr
Mode: Compatible
```
