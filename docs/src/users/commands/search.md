# search

Search ADRs for matching content.

## Usage

```
adrs search [OPTIONS] <QUERY>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<QUERY>` | Search query text |

## Options

| Option | Description |
|--------|-------------|
| `-t, --title` | Search titles only |
| `-s, --status <STATUS>` | Filter by status |
| `-c, --case-sensitive` | Case-sensitive search |

## Examples

```sh
# Search all content for 'postgres'
adrs search postgres

# Search titles only
adrs search -t database

# Search accepted ADRs for 'auth'
adrs search --status accepted auth

# Case-sensitive search
adrs search -c PostgreSQL
```

## Output

Returns matching ADRs with their number, title, and status.

```
$ adrs search database
1: Use PostgreSQL for persistence (accepted)
5: Add Redis for caching (proposed)
```

## Tips

- Search is case-insensitive by default
- Searches both title and full content unless `-t` is used
- Combine with `--status` to narrow results
- Use quotes for multi-word searches: `adrs search "event sourcing"`
