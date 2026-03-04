# MADR Format

[Markdown Architectural Decision Records](https://adr.github.io/madr/) (MADR) version 4.0.0 provides a more structured format with additional metadata and sections for detailed option comparison.

## Structure

```markdown
---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
informed:
  - Dave
---

# Use PostgreSQL for Persistence

## Context and Problem Statement

We need a database for storing user data.

## Decision Drivers

* Need ACID compliance
* Team has PostgreSQL experience
* Open source preferred

## Considered Options

* PostgreSQL
* MySQL
* MongoDB

## Decision Outcome

Chosen option: "PostgreSQL", because it meets all requirements and the team is familiar with it.

### Consequences

* Good, because we have team expertise
* Bad, because it requires more infrastructure than SQLite

### Confirmation

We will confirm this decision after the first production deployment.

## Pros and Cons of the Options

### PostgreSQL

* Good, because ACID compliant
* Good, because team experience
* Neutral, because requires server setup

### MySQL

* Good, because widely used
* Bad, because different SQL dialect

### MongoDB

* Good, because flexible schema
* Bad, because not ACID compliant by default
```

## YAML Frontmatter Fields

MADR uses YAML frontmatter for structured metadata:

| Field | Required | Description |
|-------|----------|-------------|
| `status` | Yes | Decision status (proposed, accepted, deprecated, superseded) |
| `date` | Yes | Date of the decision |
| `decision-makers` | No | List of people who made the decision |
| `consulted` | No | List of people whose opinions were sought |
| `informed` | No | List of people who were informed |

See [Frontmatter](../frontmatter.md) for full field reference.

## Sections

| Section | Required | Description |
|---------|----------|-------------|
| Title | Yes | The decision as an H1 heading |
| Context and Problem Statement | Yes | Why is this decision needed? |
| Decision Drivers | No | Factors influencing the decision |
| Considered Options | No | Options that were evaluated |
| Decision Outcome | Yes | The chosen option and rationale |
| Consequences | No | Good and bad outcomes |
| Confirmation | No | How the decision will be validated |
| Pros and Cons | No | Detailed option comparison |
| More Information | No | Links and references |

## Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `number` | ADR number | `1` |
| `title` | ADR title | `Use PostgreSQL` |
| `date` | Current date | `2024-01-15` |
| `status` | Initial status | `proposed` |
| `decision_makers` | MADR decision makers list | |
| `consulted` | MADR consulted list | |
| `informed` | MADR informed list | |

## Full Template

```markdown
---
status: {{ status | lower }}
date: {{ date }}
---

# {{ title }}

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

## Minimal Template

```markdown
---
status: {{ status | lower }}
date: {{ date }}
---

# {{ title }}

## Context and Problem Statement

## Decision Outcome

### Consequences
```

## When to Use MADR Format

- Starting a new project without adr-tools dependency
- Need structured metadata (decision-makers, consulted, informed)
- Want detailed option comparison
- Building tooling that consumes ADRs programmatically
- Need machine-readable structured data

## Usage

```sh
# MADR format
adrs new --format madr "My Decision"

# With variant
adrs new --format madr --variant minimal "My Decision"

# Set as default in adrs.toml
```

```toml
[templates]
format = "madr"
```

## Related

- [Nygard Format](./nygard.md) - Simpler alternative format
- [Variants](./variants.md) - Full, minimal, bare variants
- [Frontmatter](../frontmatter.md) - YAML frontmatter reference
- [NextGen Mode](../modes/nextgen.md) - Mode that enables MADR features

> **Related:** [ADR-0006: YAML Frontmatter for Metadata](../adrs/0006-yaml-frontmatter-for-metadata.md)
