# config

Show or manage configuration.

## Usage

```
adrs config [COMMAND]
```

## Subcommands

| Command | Description |
|---------|-------------|
| `show` | Show resolved configuration (default) |
| `migrate` | Migrate configuration between formats |

## Global Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

---

## config show

Show the current resolved configuration.

### Usage

```
adrs config show [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Show all configuration layers and their sources |

### Examples

#### Basic Usage

```sh
adrs config
# or
adrs config show
```

Output:

```
Project root: /home/user/myproject
Config source: adrs.toml
ADR directory: doc/adr
Full path: /home/user/myproject/doc/adr
Mode: Compatible
```

#### Verbose Output

```sh
adrs config show --verbose
```

Output:

```
Project root: /home/user/myproject
Config source: adrs.toml
ADR directory: doc/adr
Full path: /home/user/myproject/doc/adr
Mode: NextGen

Configuration layers (highest to lowest priority):

Layer 1: Environment variables
  (not set)

Layer 2: adrs.toml
  adr_dir = "doc/adr"
  mode = "ng"

Layer 3: .adr-dir (legacy)
  (not found)

Layer 4: Global config
  ~/.config/adrs/config.toml (not found)

Layer 5: Defaults
  adr_dir = "doc/adr"
  mode = "compatible"
```

---

## config migrate

Migrate configuration between formats.

### Usage

```
adrs config migrate --to <FORMAT> [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--to <FORMAT>` | Target format: `toml` or `adr-dir` |
| `--dry-run` | Preview migration without writing files |

### Examples

#### Convert .adr-dir to adrs.toml

```sh
adrs config migrate --to toml
```

#### Preview Migration

```sh
adrs config migrate --to toml --dry-run
```

Output:

```
Would create: /home/user/myproject/adrs.toml

adr_dir = "doc/adr"
mode = "compatible"

[templates]


Note: You may want to remove .adr-dir after migration: /home/user/myproject/.adr-dir
```

#### Convert to Legacy Format

```sh
adrs config migrate --to adr-dir
```

> **Warning:** Converting to `.adr-dir` is lossy. Only the `adr_dir` setting is preserved; mode, templates, and other settings will be lost.

---

## Config Sources

| Source | Priority | Description |
|--------|----------|-------------|
| Environment variables | 1 (highest) | `ADR_DIRECTORY` overrides |
| `adrs.toml` | 2 | Project TOML configuration |
| `.adr-dir` | 3 | Legacy adr-tools format |
| Global config | 4 | `~/.config/adrs/config.toml` |
| Defaults | 5 (lowest) | Built-in defaults |

## Related

- [Configuration](../configuration.md) - Configuration options
- [init](./init.md) - Initialize a repository
