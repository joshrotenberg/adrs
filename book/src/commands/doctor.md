# doctor

Check the health of your ADR repository.

## Usage

```
adrs doctor [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--ng` | No-op for `doctor` (prints a note; see below) |
| `--ignore <RULE>` | Ignore a rule by ID or name (repeatable); merged with `[doctor].ignore` in `adrs.toml` |
| `--warnings-as-errors` | Exit with status 1 if there are warnings, not just errors |
| `-C, --cwd <DIR>` | Working directory |
| `-h, --help` | Print help |

## Description

Runs diagnostic checks on your ADR repository and reports any issues found.

The global `--ng` flag has no effect on `doctor`. Lint rules detect each ADR's
format (Nygard or MADR) from the file itself, so the repository mode does not
change which checks run. Passing `--ng doctor` prints a note to that effect
rather than ignoring the flag silently.

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
No issues found. Your ADR repository is healthy!
```

### Repository with Issues

```sh
adrs doctor
```

Output:

```
error: [ADR012] doc/adr/0003-duplicate.md: Duplicate ADR number 2. Also used in: doc/adr/0002-use-postgresql.md
warning: [ADR009] Filename number (0003) does not match title number (2) [doc/adr/0003-duplicate.md:1]

Found 1 error(s), 1 warning(s), 0 info(s)
```

Each line has the form `<severity>: [<rule ID>] <message> [<location>]`. Rule
IDs like `ADR009` and `ADR012` map to the checks below.

## Severity Levels

| Level | Description |
|-------|-------------|
| info | Informational, no action needed |
| warning | Potential issue, but not critical |
| error | Problem that should be fixed |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | No issues, or only warnings/info (default) |
| 1 | One or more errors, or warnings with `--warnings-as-errors` / `warnings_as_errors = true` |

By default, warnings alone do not fail the check; only errors do. Pass
`--warnings-as-errors` (or set `warnings_as_errors = true` in `[doctor]`) to
also fail on warnings. This allows using `doctor` in CI pipelines:

```yaml
- name: Check ADR health
  run: adrs doctor
```

## Configuration

`doctor` reads a `[doctor]` section from `adrs.toml`:

```toml
[doctor]
# Rule IDs or rule names to suppress (matched case-insensitively)
ignore = ["ADR011"]

# Exit with status 1 if there are warnings, not just errors
warnings_as_errors = false
```

`--ignore` flags on the command line merge with (do not replace) `[doctor].ignore`
from config, so you can suppress an extra rule for a single run without editing
`adrs.toml`. `--warnings-as-errors` on the command line ORs with
`[doctor].warnings_as_errors`, so either one being set is enough to make warnings
fail the check.

## Pre-commit Hook

`adrs` ships a [pre-commit](https://pre-commit.com) hook (also compatible
with [prek](https://prek.j178.dev)) that runs `doctor` whenever a markdown
file changes. Add it to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/joshrotenberg/adrs
    rev: v0.9.0
    hooks:
      - id: adrs-doctor
```

The hook uses `language: system`, so it expects `adrs` to already be on
`PATH`. See [Installation](../installation.md) for ways to install it
(`cargo install adrs`, a release binary, Homebrew, etc.).

The hook triggers on any staged `.md` file but always checks the whole
repository, since `doctor`'s checks (numbering, links, superseded status)
are repository-wide. If your ADRs live outside the default directory, scope
the trigger further with `files:` in your own config, e.g.:

```yaml
      - id: adrs-doctor
        files: ^doc/adr/.*\.md$
```

## Related

- [list](./list.md) - List ADRs
- [link](./link.md) - Fix broken links
