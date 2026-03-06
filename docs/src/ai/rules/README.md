# AI Agent Rules

Behavioral guidelines for AI agents working with `adrs`.

## Core Rules

### Rule 1: Human Review Required

All ADRs created by AI agents MUST be set to `proposed` status. Final acceptance requires human review.

### Rule 2: Search Before Creating

Before creating a new ADR, always search for existing related decisions to avoid duplication.

### Rule 3: Preserve Context

When updating ADRs, preserve existing content unless explicitly asked to modify it.

### Rule 4: Link Related Decisions

When creating or updating ADRs, identify and create links to related decisions.

### Rule 5: Validate Before Completing

After creating or modifying an ADR, validate it to ensure completeness.

## Content Guidelines

### Titles

- Use imperative mood: "Use PostgreSQL" not "Using PostgreSQL"
- Be specific: "Use PostgreSQL for user data" not "Database choice"
- Keep concise: Under 60 characters

### Context Section

- Explain why this decision is needed
- Describe the current situation
- List constraints and requirements

### Decision Section

- State the decision clearly
- Explain the rationale
- Reference alternatives considered

### Consequences Section

- List positive outcomes
- List negative trade-offs
- Identify follow-up actions

## Safety Rules

### Do Not

- Delete existing ADRs
- Change status to `accepted` without human approval
- Modify ADRs without understanding context
- Create duplicate decisions

### Always

- Validate inputs before operations
- Report errors clearly
- Maintain audit trail
- Respect repository boundaries

## See Also

- [Skills](../skills/README.md) - Agent capabilities
- [MCP Tools](../../developers/mcp/tools/README.md) - Available tools
