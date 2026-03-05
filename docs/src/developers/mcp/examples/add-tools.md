# Adding MCP Tools

This guide walks through adding a new tool to the MCP server.

## Step 1: Define Tool Schema

In `crates/adrs-mcp/src/tools/`:

```rust
// mytool.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolInput {
    /// Description of parameter
    pub param: String,

    /// Optional parameter
    #[serde(default)]
    pub optional: Option<bool>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MyToolOutput {
    pub result: String,
}
```

## Step 2: Implement Tool Handler

```rust
// mytool.rs (continued)
use adrs_core::Repository;
use crate::error::McpError;

pub async fn handle(input: MyToolInput, repo: &Repository) -> Result<MyToolOutput, McpError> {
    // Implementation using adrs-core
    let result = do_something(&input.param, repo)?;

    Ok(MyToolOutput { result })
}

fn do_something(param: &str, repo: &Repository) -> Result<String, McpError> {
    // Use repository operations
    Ok(format!("Processed: {}", param))
}
```

## Step 3: Register Tool

In `crates/adrs-mcp/src/tools/mod.rs`:

```rust
mod mytool;

pub fn register_tools(router: &mut ToolRouter) {
    // ... existing tools ...

    router.register(
        "my_tool",
        "Description of what my tool does",
        mytool::handle,
    );
}
```

## Step 4: Add Tests

```rust
// tests/mytool_test.rs
use adrs_mcp::tools::mytool::{MyToolInput, handle};
use tempfile::tempdir;
use adrs_core::Repository;

#[tokio::test]
async fn test_mytool_basic() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path(), None, false).unwrap();

    let input = MyToolInput {
        param: "test".to_string(),
        optional: None,
    };

    let result = handle(input, &repo).await.unwrap();
    assert_eq!(result.result, "Processed: test");
}
```

## Step 5: Document Tool

Create `docs/src/developers/mcp/tools/my_tool.md`:

```markdown
# my_tool

Description of what this tool does.

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `param` | string | Yes | Description |
| `optional` | boolean | No | Description |

## Response

| Field | Type | Description |
|-------|------|-------------|
| `result` | string | Description |

## Example

\`\`\`json
{
  "name": "my_tool",
  "arguments": {
    "param": "value"
  }
}
\`\`\`
```

## Design Guidelines

### Safe Defaults

- Write operations should not delete data
- New ADRs should be created with `proposed` status
- Validate inputs before operations

### Error Handling

- Return structured errors with codes
- Don't expose internal paths
- Provide actionable messages

### Performance

- Read operations should complete in < 100ms
- Write operations should complete in < 500ms

## See Also

- [Tools Reference](../tools/README.md)
- [Requirements](../requirements/README.md)
