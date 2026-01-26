# export

Export ADRs to different formats for integration with other tools.

## JSON Export

Export ADRs to JSON-ADR format, a machine-readable interchange format.

```bash
adrs export json                  # All ADRs as JSON
adrs export json --pretty         # Pretty-printed output
adrs export json 5                # Single ADR by number
adrs export json -d path/to/adrs  # Export from directory (no repo needed)
```

### Options

| Option | Description |
|--------|-------------|
| `--pretty`, `-p` | Pretty-print the JSON output |
| `--dir`, `-d` | Export from a directory without requiring an initialized repository |
| `--metadata-only` | Export metadata without content fields (context, decision, consequences) |
| `--base-url <URL>` | Set base URL for `source_uri` field (for federation) |

### Output Format

```json
{
  "$schema": "https://raw.githubusercontent.com/joshrotenberg/adrs/main/schema/json-adr/v1.json",
  "version": "1.0.0",
  "generated_at": "2024-01-15T10:30:00Z",
  "tool": {
    "name": "adrs",
    "version": "0.5.0"
  },
  "repository": {
    "adr_directory": "doc/adr"
  },
  "adrs": [
    {
      "number": 1,
      "title": "Record architecture decisions",
      "status": "Accepted",
      "date": "2024-01-15",
      "context": "We need to record architectural decisions...",
      "decision": "We will use ADRs...",
      "consequences": "See Michael Nygard's article...",
      "links": [],
      "path": "doc/adr/0001-record-architecture-decisions.md"
    }
  ]
}
```

### Federation Support

Use `--base-url` to add `source_uri` fields for referencing ADRs across repositories:

```bash
adrs export json --metadata-only --base-url "https://github.com/org/repo/blob/main/doc/adr"
```

This produces lightweight exports that reference the source files:

```json
{
  "number": 1,
  "title": "Use PostgreSQL",
  "status": "Accepted",
  "date": "2024-01-15",
  "source_uri": "https://github.com/org/repo/blob/main/doc/adr/0001-use-postgresql.md"
}
```

## Use Cases

- **Integration**: Feed ADRs into documentation systems or dashboards
- **Backup/Migration**: Export ADRs for archival or moving between projects
- **CI/CD**: Validate ADRs exist for certain features in pipelines
- **AI Context**: Provide codebase context to LLMs
- **Federation**: Create organization-wide ADR indexes across repositories
