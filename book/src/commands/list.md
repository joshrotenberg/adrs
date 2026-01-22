# list

List all Architecture Decision Records.

## Usage

```
adrs list [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Lists all ADRs in the repository, showing their number, title, and status.

## Examples

### Basic Usage

```sh
adrs list
```

Output:

```
1. Record architecture decisions [Accepted]
2. Use PostgreSQL for persistence [Proposed]
3. API versioning strategy [Accepted]
4. Use Redis for caching [Superseded]
```

### From a Subdirectory

```sh
cd src/
adrs list
```

`adrs` automatically finds the project root.

### Specify Working Directory

```sh
adrs list -C /path/to/project
```

## Output Format

Each line shows:
- ADR number
- Title
- Status in brackets

ADRs are sorted by number.

## Related

- [new](./new.md) - Create a new ADR
- [edit](./edit.md) - Edit an ADR from the list
