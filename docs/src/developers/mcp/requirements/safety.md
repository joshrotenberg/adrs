# Safety Requirements

<!-- toc -->

## Safe Defaults

### MCP-SAFE-1: Proposed Status

**Requirement:** `create_adr` MUST create with `proposed` status.

**Rationale:** AI-created ADRs require human review before acceptance.

### MCP-SAFE-2: No Deletion

**Requirement:** Write operations MUST NOT delete files.

**Rationale:** Prevent accidental data loss.

### MCP-SAFE-3: Input Validation

**Requirement:** All inputs MUST be validated before operations.

**Checks:**
- ADR numbers must be positive integers
- Titles must be non-empty
- Status must be valid value
- Links must reference existing ADRs

## Error Handling

### MCP-SAFE-4: Structured Errors

**Requirement:** Errors MUST return structured responses.

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "ADR 5 not found",
    "details": {}
  }
}
```

### MCP-SAFE-5: No Path Exposure

**Requirement:** Error messages MUST NOT expose internal paths.

**Bad:**
```json
{
  "error": "Failed to read /home/user/project/doc/adr/0001.md"
}
```

**Good:**
```json
{
  "error": "ADR 1 not found"
}
```

### MCP-SAFE-6: Graceful Degradation

**Requirement:** Partial failures SHOULD NOT corrupt repository.

- File operations should be atomic
- Interrupted operations should not leave partial state

## Authorization

### MCP-SAFE-7: Repository Scope

**Requirement:** Operations MUST be scoped to configured repository.

- MUST NOT access files outside ADR directory
- MUST NOT execute shell commands
- MUST NOT access network (except HTTP transport)

## Audit

### MCP-SAFE-8: Logging

**Requirement:** Tool invocations SHOULD be logged.

Log should include:
- Timestamp
- Tool name
- Input parameters (sanitized)
- Success/failure
- Duration

## See Also

- [Tool Requirements](./tools.md)
- [Performance Requirements](./performance.md)
