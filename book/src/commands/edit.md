# edit

Edit an existing Architecture Decision Record.

## Usage

```
adrs edit [OPTIONS] <ADR>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<ADR>` | ADR to edit (number or search term) |

## Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Opens an existing ADR in your editor. The ADR can be specified by:
- Number (e.g., `1`, `12`)
- Partial title match (fuzzy search)

## Examples

### By Number

```sh
adrs edit 1
```

Opens `0001-record-architecture-decisions.md` in your editor.

### By Title

```sh
adrs edit postgresql
```

Finds and opens the ADR with "postgresql" in its title.

### Fuzzy Matching

```sh
adrs edit "database"
```

Opens the best matching ADR containing "database".

## Editor

Uses the `$EDITOR` environment variable. See [new](./new.md#editor) for configuration.

## Related

- [new](./new.md) - Create a new ADR
- [list](./list.md) - List ADRs to find the one to edit
