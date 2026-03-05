# Template Variants

Both Nygard and MADR formats support four variants that control the level of detail and guidance in generated templates.

## Variant Comparison

| Variant | All Sections | Guidance Comments | Best For |
|---------|--------------|-------------------|----------|
| `full` | Yes | Yes | New users, learning |
| `minimal` | Core only | No | Experienced users |
| `bare` | Yes | No | Custom workflows |
| `bare-minimal` | Core only | No | Maximum brevity |

## Usage

```sh
# Specify variant with --variant flag
adrs new --variant full "My Decision"
adrs new --variant minimal "My Decision"
adrs new --variant bare "My Decision"
adrs new --variant bare-minimal "My Decision"

# Combine with format
adrs new --format madr --variant minimal "My Decision"
```

## Nygard Variants

### Full (default)

All sections with guidance comments:

```markdown
# 1. Use PostgreSQL

Date: 2024-01-15

## Status

Proposed

## Context

<!-- Describe the issue motivating this decision and any context -->

## Decision

<!-- What is the change that we're proposing and/or doing? -->

## Consequences

<!-- What becomes easier or more difficult to do because of this change? -->
```

### Minimal

Essential sections only, no comments:

```markdown
# 1. Use PostgreSQL

Date: 2024-01-15

## Status

Proposed

## Context

## Decision

## Consequences
```

### Bare

All sections, no guidance:

```markdown
# 1. Use PostgreSQL

Date: 2024-01-15

## Status

Proposed

## Context

## Decision

## Consequences
```

### Bare-Minimal

Core sections only, empty:

```markdown
# 1. Use PostgreSQL

Date: 2024-01-15

## Status

Proposed

## Context

## Decision

## Consequences
```

## MADR Variants

### Full (default)

All sections with guidance:

```markdown
---
status: proposed
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

<!-- Describe the context and problem statement -->

## Decision Drivers

<!-- List the decision drivers -->

## Considered Options

<!-- List the options considered -->

## Decision Outcome

Chosen option: "", because ...

### Consequences

* Good, because ...
* Bad, because ...

### Confirmation

<!-- How will this decision be confirmed? -->

## Pros and Cons of the Options

### Option 1

* Good, because ...
* Bad, because ...

## More Information

<!-- Links, references, etc. -->
```

### Minimal

Essential sections only:

```markdown
---
status: proposed
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

## Decision Outcome

### Consequences
```

### Bare

All sections, no guidance:

```markdown
---
status: proposed
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

## Decision Drivers

## Considered Options

## Decision Outcome

### Consequences

### Confirmation

## Pros and Cons of the Options

## More Information
```

### Bare-Minimal

Core sections only, empty:

```markdown
---
status: proposed
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

## Decision Outcome

### Consequences
```

## Default Configuration

Set your preferred variant in `adrs.toml`:

```toml
[templates]
variant = "minimal"
```

## Choosing a Variant

| Scenario | Recommended Variant |
|----------|---------------------|
| New to ADRs | `full` |
| Established team | `minimal` |
| Automated tooling | `bare` or `bare-minimal` |
| Quick drafts | `bare-minimal` |

## Related

- [Nygard Format](./nygard.md)
- [MADR Format](./madr.md)
- [Templates Overview](./README.md)
