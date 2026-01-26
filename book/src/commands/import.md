# import

Import ADRs from different formats into your repository.

## JSON Import

Import ADRs from JSON-ADR format files.

```bash
adrs import json exported.json              # Import to repository
adrs import json exported.json --dry-run    # Preview without writing
adrs import json - < exported.json          # Read from stdin
```

### Options

| Option | Description |
|--------|-------------|
| `--dir`, `-d` | Import to a directory without requiring an initialized repository |
| `--overwrite`, `-o` | Overwrite existing files with the same number |
| `--renumber`, `-r` | Renumber ADRs starting from next available number |
| `--append` | Alias for `--renumber` |
| `--dry-run` | Preview import without writing files |
| `--ng` | Use NextGen mode with YAML frontmatter |

## Merging ADR Sets with --renumber/--append

The `--renumber` option (also available as `--append`) enables merging ADRs from different sources without number conflicts. It automatically:

1. Finds the next available number in the target directory
2. Renumbers all imported ADRs sequentially from that number
3. Updates cross-references between imported ADRs

### Example: Merging Projects

If your target repository has ADRs 1-5:

```bash
# Imported ADRs become 6, 7, 8...
adrs import json external-adrs.json --renumber

# Or use the --append alias for clarity
adrs import json external-adrs.json --append
```

### Example: Migration Scenario

Combine with `--dir` for migrating ADRs:

```bash
adrs import json acquired-project.json --dir doc/adr --append
```

### Preview Before Importing

Use `--dry-run` to see what would happen:

```bash
adrs import json external.json --append --dry-run
```

Output shows the renumber mapping:
```
Would import 3 ADRs:
  ADR 1 -> ADR 6 (0006-use-postgresql.md)
  ADR 2 -> ADR 7 (0007-use-redis.md)
  ADR 3 -> ADR 8 (0008-api-design.md)

Cross-references updated:
  ADR 7 link to ADR 1 -> ADR 6
```

### Link Handling

When renumbering, internal links between imported ADRs are automatically updated. For example:

- Source ADR 2 links to ADR 1 with "Supersedes"
- After renumbering to 7 and 6, the link now points to ADR 6

Links to ADRs outside the imported set generate warnings:
```
Warning: ADR 7 links to ADR 5 which is not in the import set
```

## Input Formats

The import command accepts multiple JSON-ADR formats:

**Bulk export** (from `adrs export json`):
```json
{
  "version": "1.0.0",
  "adrs": [...]
}
```

**Single ADR wrapper**:
```json
{
  "version": "1.0.0",
  "adr": { ... }
}
```

**Bare ADR object**:
```json
{
  "number": 1,
  "title": "...",
  "status": "...",
  ...
}
```

## Use Cases

- **Project mergers**: Combine ADRs when acquiring or merging codebases
- **Migration**: Move ADRs between repositories
- **Backup restoration**: Restore from JSON exports
- **Cross-team sharing**: Import relevant decisions from other teams
