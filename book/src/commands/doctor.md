# doctor

Check the health of your ADR repository.

## Usage

```
adrs doctor [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--ng` | Use NextGen mode |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Runs diagnostic checks on your ADR repository and reports any issues found.

## Checks Performed

| Check | Description |
|-------|-------------|
| File Naming | ADR files follow the `NNNN-title.md` pattern |
| Duplicate Numbers | No two ADRs have the same number |
| Numbering Gaps | Sequential numbering without gaps |
| Broken Links | All referenced ADRs exist |
| Superseded Status | Superseded ADRs have corresponding links |
| Parse Errors | All ADRs can be parsed correctly |

## Examples

### Healthy Repository

```sh
adrs doctor
```

Output:

```
Checking ADR repository health...

[OK] File naming: All 5 ADRs follow naming convention
[OK] Duplicate numbers: No duplicates found
[OK] Numbering gaps: Numbers are sequential
[OK] Broken links: All links are valid
[OK] Superseded status: All superseded ADRs have links
[OK] Parse errors: All ADRs parsed successfully

Health check passed!
```

### Repository with Issues

```sh
adrs doctor
```

Output:

```
Checking ADR repository health...

[OK] File naming: All 5 ADRs follow naming convention
[WARN] Numbering gaps: Gap after ADR 3 (next is 5)
[ERROR] Broken links: ADR 4 links to non-existent ADR 99
[WARN] Superseded status: ADR 2 is superseded but has no "Superseded by" link

Health check found 3 issue(s)
```

## Severity Levels

| Level | Description |
|-------|-------------|
| OK | Check passed |
| WARN | Potential issue, but not critical |
| ERROR | Problem that should be fixed |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | All checks passed |
| 1 | One or more checks failed |

This allows using `doctor` in CI pipelines:

```yaml
- name: Check ADR health
  run: adrs doctor
```

## Related

- [list](./list.md) - List ADRs
- [link](./link.md) - Fix broken links
