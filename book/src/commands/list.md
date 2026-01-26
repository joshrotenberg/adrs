# list

List Architecture Decision Records with optional filtering.

## Usage

```bash
adrs list [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `-l, --long` | Show detailed output (number, status, date, title) |
| `-s, --status <STATUS>` | Filter by status (e.g., proposed, accepted, deprecated, superseded) |
| `--since <DATE>` | Show ADRs from this date onwards (YYYY-MM-DD) |
| `--until <DATE>` | Show ADRs up to this date (YYYY-MM-DD) |
| `--decider <NAME>` | Filter by decision maker (MADR format) |
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |

## Examples

### Basic Usage

```bash
adrs list
```

Output (paths):
```
doc/adr/0001-record-architecture-decisions.md
doc/adr/0002-use-postgresql.md
doc/adr/0003-api-versioning.md
```

### Detailed Output

```bash
adrs list -l
```

Output:
```
   1  Accepted    2024-01-15  Record architecture decisions
   2  Accepted    2024-01-20  Use PostgreSQL for persistence
   3  Proposed    2024-02-01  API versioning strategy
   4  Superseded  2024-01-10  Use MySQL for persistence
```

### Filter by Status

```bash
# Show accepted ADRs
adrs list --status accepted -l

# Show ADRs needing review
adrs list --status proposed -l

# Show superseded ADRs
adrs list --status superseded -l
```

### Filter by Date

```bash
# ADRs from 2024 onwards
adrs list --since 2024-01-01 -l

# ADRs before 2024
adrs list --until 2023-12-31 -l

# ADRs in a date range
adrs list --since 2024-01-01 --until 2024-06-30 -l
```

### Filter by Decision Maker

For ADRs using MADR format with decision-makers metadata:

```bash
# Find ADRs decided by Alice (case-insensitive substring match)
adrs list --decider alice -l

# Find ADRs by team
adrs list --decider "Security Team" -l
```

### Combined Filters

Multiple filters are AND'd together:

```bash
# Accepted ADRs from 2024 decided by Alice
adrs list --status accepted --since 2024-01-01 --decider alice -l
```

## Output Format

**Default**: File paths, one per line

**Long format (`-l`)**: Columns showing:
- ADR number (4 chars)
- Status (12 chars)
- Date (YYYY-MM-DD)
- Title

ADRs are sorted by number.

## Related

- [new](./new.md) - Create a new ADR
- [edit](./edit.md) - Edit an ADR from the list
- [status](./status.md) - Change an ADR's status
