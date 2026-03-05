# AI Agent Hooks

Integration hooks for AI agents working with `adrs`.

## Overview

Hooks allow AI agents to integrate with workflows and trigger actions based on events.

## Available Hooks

### Pre-Create Hook

Triggered before creating a new ADR.

**Use cases:**
- Validate title format
- Check for duplicates
- Suggest related ADRs

### Post-Create Hook

Triggered after creating a new ADR.

**Use cases:**
- Notify stakeholders
- Update documentation index
- Trigger review workflow

### Status Change Hook

Triggered when ADR status changes.

**Use cases:**
- Notify on acceptance
- Archive deprecated decisions
- Update dependency tracking

## Integration Examples

### GitHub Actions

```yaml
on:
  push:
    paths:
      - 'doc/adr/*.md'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install adrs
        run: cargo install adrs
      - name: Validate ADRs
        run: adrs doctor --json
```

### Claude Code Hooks

In `.claude/settings.json`:

```json
{
  "hooks": {
    "post-tool-use": {
      "adrs": {
        "create_adr": "adrs doctor"
      }
    }
  }
}
```

## Custom Hooks

Hooks can be implemented via:

1. **Shell scripts**: Executed after operations
2. **MCP notifications**: Sent to connected clients
3. **Webhooks**: HTTP callbacks (future)

## See Also

- [Skills](../skills/README.md) - Agent capabilities
- [Rules](../rules/README.md) - Behavioral guidelines
