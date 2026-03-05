# Tool Requirements

<!-- toc -->

## Read Tools

### MCP-TOOL-1: list_adrs

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter by status |
| `tag` | string | No | Filter by tag |

**Returns:** Array of ADR summaries

### MCP-TOOL-2: get_adr

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `number` | integer | Yes | ADR number |

**Returns:** Full ADR content including sections

### MCP-TOOL-3: search_adrs

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Search text |
| `title_only` | boolean | No | Only search titles |

**Returns:** Array of matching ADRs

## Write Tools

### MCP-TOOL-4: create_adr

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | ADR title |
| `context` | string | No | Context section |
| `decision` | string | No | Decision section |
| `consequences` | string | No | Consequences section |
| `supersedes` | integer | No | ADR to supersede |

**Requirements:**
- MUST create with `proposed` status
- MUST auto-assign next number
- MUST create supersession link if specified

### MCP-TOOL-5: update_status

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `number` | integer | Yes | ADR number |
| `status` | string | Yes | New status |
| `superseded_by` | integer | No | Required for superseded |

**Requirements:**
- MUST validate ADR exists
- MUST create link when superseding

### MCP-TOOL-6: link_adrs

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source` | integer | Yes | Source ADR |
| `target` | integer | Yes | Target ADR |
| `link_type` | string | Yes | Link type |

**Requirements:**
- MUST create bidirectional links
- MUST validate both ADRs exist

## Analysis Tools

### MCP-TOOL-7: validate_adr

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `number` | integer | Yes | ADR to validate |

**Returns:** Validation results with severity levels

### MCP-TOOL-8: compare_adrs

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `adr1` | integer | Yes | First ADR |
| `adr2` | integer | Yes | Second ADR |

**Returns:** Structural comparison

### MCP-TOOL-9: suggest_tags

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `number` | integer | Yes | ADR to analyze |

**Returns:** Suggested tags with confidence

## See Also

- [Safety Requirements](./safety.md)
- [Performance Requirements](./performance.md)
