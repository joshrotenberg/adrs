# Project Requirements

## What Are Requirements?

Requirements are documented expectations for system behavior. They fall into two categories:

### Functional Requirements (FR)

**What the system must do.**

- Features and capabilities
- Input/output behavior
- Business rules
- Data processing

Example: "The tool MUST be able to create ADRs with auto-numbered filenames."

### Non-Functional Requirements (NFR)

**How well the system must do it.**

- Performance (speed, throughput)
- Reliability (uptime, error handling)
- Usability (ease of use, documentation)
- Maintainability (code quality, testability)
- Security (data protection, access control)

Example: "The tool MUST list 1000 ADRs in under 1 second."

## Why Requirements Matter

### For Users

- Know what to expect
- Understand limitations
- Evaluate fitness for purpose

### For Developers

- Guide implementation decisions
- Define acceptance criteria
- Prioritize work

### For Reviewers

- Validate changes meet expectations
- Catch regressions
- Ensure consistency

## When to Use Requirements

### During Planning

- Define scope
- Identify priorities
- Estimate effort

### During Implementation

- Guide design decisions
- Validate completeness
- Write tests

### During Review

- Verify behavior
- Check edge cases
- Ensure quality

## Requirements in This Project

| Document | Scope |
|----------|-------|
| [Functional](./functional.md) | What the tool does |
| [Non-Functional](./non-functional.md) | Quality attributes |
| [Library Requirements](../../developers/lib/requirements/README.md) | Library API |
| [CLI Requirements](../../developers/cli/requirements/README.md) | CLI behavior |
| [MCP Requirements](../../developers/mcp/requirements/README.md) | MCP server |

## See Also

- [ADRs](../../reference/adrs/README.md) - Design decisions
- [Contributing](../../developers/contributing.md) - Development guidelines
