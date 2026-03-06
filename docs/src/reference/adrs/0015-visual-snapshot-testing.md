# 15. Visual/Snapshot Testing

Date: 2026-03-05

## Status

Accepted

## Context

CLI tools need regression testing for their output format. Traditional unit tests verify logic,
but don't catch unintended changes to:

- Help text formatting and wording
- Error message presentation
- Output structure (columns, alignment, colors)
- Version-specific output differences

Manual verification is error-prone and doesn't scale. We need automated testing that:

- Captures expected output as files (snapshots)
- Compares actual output against snapshots
- Makes intentional changes explicit (requires updating snapshots)
- Works with any test runner or build tool

## Decision

Adopt visual/snapshot testing for CLI output verification:

### Directory Structure

Store expected output in `tests/visual/<command>/<case>.<ext>`:

```
tests/visual/
├── help/
│   ├── default.txt       # Main help output
│   ├── init.txt          # init --help
│   └── new.txt           # new --help
├── list/
│   ├── empty.txt         # No ADRs
│   ├── single.txt        # One ADR
│   └── multiple.txt      # Many ADRs
├── export/
│   ├── json.json         # JSON export
│   └── json-pretty.json  # Pretty-printed JSON
└── error/
    ├── not-found.txt     # ADR not found error
    └── invalid-id.txt    # Invalid ID format
```

### File Extensions

| Extension | Use Case |
|-----------|----------|
| `.txt` | Plain text output |
| `.json` | JSON output (validated) |
| `.md` | Markdown output |
| `.ansi` | Output with ANSI color codes |

### Test Workflow

1. **Run test**: Execute command, capture output
2. **Compare**: Diff actual vs expected
3. **Pass**: Outputs match exactly
4. **Fail**: Show diff, suggest update command

### Update Workflow

When output intentionally changes:

1. Run update command to regenerate snapshots
2. Review diff in version control
3. Commit updated snapshots with explanation

### Implementation Patterns

Tests should be tool-agnostic. Example shell implementation:

```bash
# Run command and capture output
ACTUAL=$(mktemp)
$CLI --help > "$ACTUAL" 2>&1

# Compare against expected
if diff -u "$EXPECTED" "$ACTUAL"; then
    echo "✓ Test passed"
else
    echo "✗ Test failed"
    echo "To update: cp $ACTUAL $EXPECTED"
    exit 1
fi
```

### Filtering Dynamic Content

Some output contains dynamic content that shouldn't be compared:

| Content | Strategy |
|---------|----------|
| Timestamps | Filter with sed/awk before compare |
| Absolute paths | Replace with placeholders |
| Version numbers | Use regex patterns |
| Colors | Strip ANSI codes or use `.ansi` extension |

Example filter:

```bash
# Strip version numbers before comparing
sed 's/v[0-9]\+\.[0-9]\+\.[0-9]\+/vX.Y.Z/g' "$ACTUAL" > "$FILTERED"
```

## Consequences

### Positive

- Catches unintended output changes automatically
- Documents expected output as test fixtures
- Easy to update when changes are intentional
- Works with any language/framework
- Readable diffs in code review

### Negative

- Snapshots can be noisy in version control
- Requires discipline to review snapshot updates
- Dynamic content needs filtering
- Large outputs increase repository size

### Neutral

- Trade-off between snapshot granularity and maintenance
- Team must agree on what constitutes "visual" vs "behavioral" changes

## References

- [Jest Snapshot Testing](https://jestjs.io/docs/snapshot-testing)
- [insta (Rust snapshot library)](https://insta.rs/)
- [cargo-insta](https://crates.io/crates/cargo-insta)
