# CLI Requirements

Requirements for the `adrs` CLI application.

## Command Requirements

### CLI-1: Core Commands

| Command | Description | Priority |
|---------|-------------|----------|
| `init` | Initialize ADR repository | P0 |
| `new` | Create new ADR | P0 |
| `list` | List ADRs | P0 |
| `edit` | Edit existing ADR | P0 |
| `status` | Change ADR status | P0 |
| `link` | Link two ADRs | P0 |
| `search` | Search ADRs | P1 |
| `export` | Export to JSON-ADR | P1 |
| `import` | Import from JSON-ADR | P1 |
| `generate` | Generate TOC/graph/book | P1 |
| `doctor` | Health checks | P1 |
| `config` | Show configuration | P2 |
| `template` | Manage templates | P2 |
| `completions` | Shell completions | P2 |

### CLI-2: Global Options

```
--ng              Enable NextGen mode
-C, --cwd <DIR>   Change working directory
-h, --help        Show help
-V, --version     Show version
```

### CLI-3: Output Formats

- Default: Human-readable text
- `--json`: JSON output for scripting
- `--quiet`: Minimal output

## Compatibility Requirements

### CLI-4: adr-tools Compatibility

- MUST accept adr-tools command syntax
- MUST produce compatible file output
- MUST read existing repositories

### CLI-5: Cross-Platform

- MUST work on Linux, macOS, Windows
- MUST handle path separators correctly
- MUST use appropriate config directories

## User Experience

### CLI-6: Error Messages

- MUST be actionable
- MUST include context (file, line)
- MUST suggest solutions

### CLI-7: Editor Integration

- MUST respect `$EDITOR`
- MUST fall back to common editors
- MUST handle editor exit codes

### CLI-8: Shell Completions

- MUST support bash, zsh, fish, PowerShell
- MUST complete command names
- MUST complete ADR numbers where applicable
