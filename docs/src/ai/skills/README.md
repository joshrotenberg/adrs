# AI Skills

Skills are pre-defined capabilities that AI agents can use when working with `adrs`.

## Available Skills

### [Development Skills](./development.md)

For AI agents working on the `adrs` codebase:
- Understanding project architecture
- Running tests
- Making code changes
- Creating pull requests

### [Operations Skills](./operations.md)

For AI agents using `adrs` to manage ADRs:
- Creating new decisions
- Searching and analyzing existing decisions
- Updating decision status
- Generating documentation

## Skill Format

Each skill includes:
- **Trigger**: When to use the skill
- **Context**: Required information
- **Actions**: Steps to perform
- **Output**: Expected results

## Using Skills

Skills can be invoked through:
1. Direct MCP tool calls
2. Natural language requests
3. Slash commands (if supported)

## Extending Skills

Skills can be customized for specific workflows:

```yaml
# Example custom skill definition
name: create-adr
description: Create a new ADR with context from discussion
trigger: "create adr for"
actions:
  - search_adrs: check for related decisions
  - create_adr: create with proposed status
  - link_adrs: link to related decisions
```
