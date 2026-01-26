# status

Change an ADR's status.

## Usage

```bash
adrs status <NUMBER> <STATUS>
adrs status <NUMBER> superseded --by <NUMBER>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `NUMBER` | The ADR number to update |
| `STATUS` | New status value |

## Options

| Option | Description |
|--------|-------------|
| `--by <NUMBER>` | For 'superseded' status: the ADR number that supersedes this one |

## Standard Statuses

- `proposed` - Decision is proposed but not yet accepted
- `accepted` - Decision has been accepted
- `deprecated` - Decision is deprecated but not replaced
- `superseded` - Decision has been replaced by another ADR

You can also use custom status values for your workflow (e.g., `draft`, `rejected`, `on-hold`).

## Examples

### Accept a proposed ADR

```bash
adrs status 5 accepted
```

### Deprecate an ADR

```bash
adrs status 3 deprecated
```

### Supersede an ADR

When superseding, use `--by` to create a bidirectional link:

```bash
adrs status 2 superseded --by 5
```

This updates ADR 2's status and creates links:
- ADR 2: "Superseded by [5. New Approach](0005-new-approach.md)"
- ADR 5: "Supersedes [2. Old Approach](0002-old-approach.md)"

### Custom status

```bash
adrs status 4 "on-hold"
```

## Notes

- The status command updates the ADR file in place
- For superseded status, using `--by` is recommended to maintain traceability
- Status values are case-insensitive when parsing but preserved as written
