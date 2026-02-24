//! Edge case tests for metadata updates (issue #187).
//!
//! Each test writes a realistic ADR fixture, calls `set_status()`, and asserts
//! the body is preserved byte-for-byte.

use adrs_core::{AdrStatus, Repository};
use std::fs;
use tempfile::TempDir;

/// Helper: init a NextGen repo, write fixture at ADR #2, change status, return resulting content.
fn roundtrip_ng(fixture: &str) -> String {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();

    let adr_path = repo.adr_path().join("0002-test.md");
    fs::write(&adr_path, fixture).unwrap();

    repo.set_status(2, AdrStatus::Accepted, None).unwrap();

    fs::read_to_string(&adr_path).unwrap()
}

/// Helper: init a Compatible repo, write fixture at ADR #2, change status, return resulting content.
fn roundtrip_legacy(fixture: &str) -> String {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();

    let adr_path = repo.adr_path().join("0002-test.md");
    fs::write(&adr_path, fixture).unwrap();

    repo.set_status(2, AdrStatus::Accepted, None).unwrap();

    fs::read_to_string(&adr_path).unwrap()
}

/// Extract the body (everything after the closing frontmatter delimiter).
fn frontmatter_body(content: &str) -> &str {
    let rest = content.strip_prefix("---\n").unwrap();
    let end = rest.find("\n---\n").unwrap();
    &rest[end + 4..] // skip \n---\n
}

// ========== NextGen / Frontmatter Fixtures ==========

#[test]
fn test_madr_with_spdx_headers() {
    let fixture = r#"---
# SPDX-License-Identifier: MIT
# SPDX-FileCopyrightText: 2026 Example Corp
number: 2
title: Use MADR format
date: 2026-01-15
status: proposed
---

# Use MADR format

## Context and Problem Statement

We need a standard format for our ADRs that supports rich metadata.

## Considered Options

* MADR 4.0.0
* Nygard format
* Custom format

## Decision Outcome

Chosen option: "MADR 4.0.0", because it provides structured metadata in YAML.

### Consequences

* Good, because we get machine-readable metadata
* Good, because MADR is widely adopted
* Bad, because it's more complex than Nygard

## More Information

See [MADR repository](https://adr.github.io/madr/) for the specification.
"#;

    let result = roundtrip_ng(fixture);

    assert!(result.contains("# SPDX-License-Identifier: MIT"));
    assert!(result.contains("# SPDX-FileCopyrightText: 2026 Example Corp"));
    assert!(result.contains("status: accepted"));

    // Compare bodies
    let original_body = frontmatter_body(fixture);
    let result_body = frontmatter_body(&result);
    assert_eq!(result_body, original_body, "Body was modified");
}

#[test]
fn test_madr_all_optional_sections() {
    let fixture = r#"---
number: 2
title: Use Redis for caching
date: 2026-02-01
status: proposed
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
informed:
  - Dave
  - Eve
tags:
  - caching
  - infrastructure
---

# Use Redis for caching

## Context and Problem Statement

Our API response times exceed SLA thresholds. We need a caching layer.

## Decision Drivers

* Response time must be < 100ms at p99
* Must support cache invalidation
* Team familiarity

## Considered Options

* Redis
* Memcached
* Application-level caching with `moka`

## Decision Outcome

Chosen option: "Redis", because it supports complex data types and pub/sub.

### Consequences

* Good, because it reduces database load by ~60%
* Bad, because it adds operational complexity
* Neutral, because team has moderate Redis experience

### Confirmation

We will monitor cache hit rates and p99 latency for 2 weeks.

## Pros and Cons of the Options

### Redis

* Good, because it supports sorted sets, hashes, streams
* Good, because it has built-in replication
* Bad, because single-threaded for commands
* Bad, because memory-only by default

### Memcached

* Good, because it's simpler to operate
* Good, because it's multi-threaded
* Bad, because it only supports string values
* Bad, because no built-in replication

### Application-level caching

* Good, because no additional infrastructure
* Good, because type-safe with Rust
* Bad, because not shared across instances
* Bad, because limited by process memory

## More Information

See [Redis documentation](https://redis.io/docs/) and our
[infrastructure runbook](https://internal.example.com/runbooks/redis).
"#;

    let result = roundtrip_ng(fixture);

    assert!(result.contains("status: accepted"));
    // Tags should still be present (from the parsed ADR)
    assert!(result.contains("tags:"));

    let original_body = frontmatter_body(fixture);
    let result_body = frontmatter_body(&result);
    assert_eq!(result_body, original_body, "Body was modified");
}

#[test]
fn test_frontmatter_with_custom_yaml_fields() {
    let fixture = r#"---
number: 2
title: Custom fields test
date: 2026-01-15
status: proposed
custom_field: some_value
priority: high
---

## Context

This ADR has custom YAML fields that we don't know about.

## Decision

Keep them.
"#;

    let result = roundtrip_ng(fixture);

    assert!(result.contains("status: accepted"));
    assert!(result.contains("custom_field: some_value"));
    assert!(result.contains("priority: high"));

    let original_body = frontmatter_body(fixture);
    let result_body = frontmatter_body(&result);
    assert_eq!(result_body, original_body, "Body was modified");
}

#[test]
fn test_body_with_horizontal_rules() {
    let fixture = r#"---
number: 2
title: Horizontal rules in body
date: 2026-01-15
status: proposed
---

## Context

Some context.

---

More context after a horizontal rule.

---

Even more after another rule.

## Decision

The decision.
"#;

    let result = roundtrip_ng(fixture);

    assert!(result.contains("status: accepted"));

    let original_body = frontmatter_body(fixture);
    let result_body = frontmatter_body(&result);
    assert_eq!(
        result_body, original_body,
        "Body with --- horizontal rules was modified"
    );
}

// ========== Legacy Fixtures ==========

#[test]
fn test_legacy_with_markdown_tables() {
    let fixture = r#"# 2. Database selection

Date: 2026-01-15

## Status

Proposed

## Context

We evaluated several databases:

| Database   | Latency | Cost    | Ease    |
|------------|---------|---------|---------|
| PostgreSQL | 5ms     | $$      | Medium  |
| MySQL      | 7ms     | $       | Easy    |
| MongoDB    | 3ms     | $$$     | Easy    |

## Decision

We will use PostgreSQL.

## Consequences

- We get ACID compliance
- `pg_dump` for backups
- See [migration guide](./migrations.md)
"#;

    let result = roundtrip_legacy(fixture);

    assert!(result.contains("Accepted"));
    assert!(result.contains("| PostgreSQL | 5ms     | $$      | Medium  |"));
    assert!(result.contains("`pg_dump` for backups"));
    assert!(result.contains("[migration guide](./migrations.md)"));
}

#[test]
fn test_legacy_with_code_blocks() {
    let fixture = r#"# 2. API design

## Status

Proposed

## Context

We need to define our REST API format.

## Decision

Use JSON:API format:

```json
{
  "data": {
    "type": "articles",
    "id": "1",
    "attributes": {
      "title": "Rails is Omakase"
    }
  }
}
```

## Consequences

- Standard format for all responses
- Client libraries available
"#;

    let result = roundtrip_legacy(fixture);

    assert!(result.contains("Accepted"));
    assert!(result.contains("```json"));
    assert!(result.contains("\"Rails is Omakase\""));
    assert!(result.contains("Standard format for all responses"));
}

#[test]
fn test_legacy_with_nested_lists_and_links() {
    let fixture = r#"# 2. Authentication strategy

## Status

Proposed

## Context

We need authentication for our API. Options:

1. **JWT tokens**
   - Stateless
   - See [RFC 7519](https://tools.ietf.org/html/rfc7519)
2. **Session cookies**
   - Server-side state
   - See [OWASP guide](https://owasp.org/www-community/Session_Management)
3. **OAuth 2.0**
   - Delegated auth
   - See [RFC 6749](https://tools.ietf.org/html/rfc6749)

## Decision

Use JWT with refresh tokens.

## Consequences

- Scalable across services
- Must handle token rotation
"#;

    let result = roundtrip_legacy(fixture);

    assert!(result.contains("Accepted"));
    assert!(result.contains("[RFC 7519](https://tools.ietf.org/html/rfc7519)"));
    assert!(result.contains("[OWASP guide](https://owasp.org/www-community/Session_Management)"));
    assert!(result.contains("**JWT tokens**"));
    assert!(result.contains("Must handle token rotation"));
}
