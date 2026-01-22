# config

Show the current configuration.

## Usage

```
adrs config [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Displays the current configuration, including where it was loaded from.

## Examples

### Basic Usage

```sh
adrs config
```

Output:

```
Project root: /home/user/myproject
Config source: adrs.toml
ADR directory: doc/adr
Full path: /home/user/myproject/doc/adr
Mode: Compatible
```

### Without Initialization

```sh
adrs config
```

Output when no ADR repository is found:

```
Project root: /home/user/myproject
Config source: defaults
ADR directory: doc/adr
Full path: /home/user/myproject/doc/adr
Mode: Compatible
```

## Config Sources

| Source | Description |
|--------|-------------|
| `adrs.toml` | TOML configuration file |
| `.adr-dir` | Legacy adr-tools configuration |
| `global (~/.config/adrs/config.toml)` | User-wide configuration |
| `environment` | Environment variable override |
| `defaults` | Built-in defaults |

## Related

- [Configuration](../configuration.md) - Configuration options
- [init](./init.md) - Initialize a repository
