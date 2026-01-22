# Formats

`adrs` supports two ADR formats: Nygard (the original adr-tools format) and MADR 4.0.0.

## Nygard Format

The classic format from [adr-tools](https://github.com/npryce/adr-tools), based on Michael Nygard's blog post.

### Structure

```markdown
# 1. Record architecture decisions

Date: 2024-01-15

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as described by Michael Nygard.

## Consequences

See Michael Nygard's article for a detailed description.
```

### Sections

| Section | Description |
|---------|-------------|
| Title | Number and title as H1 heading |
| Date | When the decision was made |
| Status | Proposed, Accepted, Deprecated, Superseded |
| Context | Why is this decision needed? |
| Decision | What was decided? |
| Consequences | What are the implications? |

## MADR 4.0.0 Format

[Markdown Architectural Decision Records](https://adr.github.io/madr/) version 4.0.0 provides a more structured format with additional metadata.

### Structure

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

### YAML Frontmatter Fields

| Field | Description |
|-------|-------------|
| `status` | Decision status |
| `date` | Date of the decision |
| `decision-makers` | List of people who made the decision |
| `consulted` | List of people whose opinions were sought |
| `informed` | List of people who were informed of the decision |

### Sections

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

## Choosing a Format

| Use Nygard when... | Use MADR when... |
|--------------------|------------------|
| Migrating from adr-tools | Starting a new project |
| Simplicity is preferred | You need structured metadata |
| Team is familiar with it | You want decision-maker tracking |
| | You need detailed option comparison |

## Specifying Format

### Per-command

```sh
adrs new --format madr "Use PostgreSQL"
adrs new --format nygard "Use PostgreSQL"
```

### In Configuration

```toml
[templates]
format = "madr"
```

## Template Variants

Both formats support three variants:

| Variant | Description |
|---------|-------------|
| `full` | All sections with guidance comments |
| `minimal` | Essential sections only |
| `bare` | Bare structure, no comments |

```sh
adrs new --format madr --variant minimal "Use PostgreSQL"
```

See [Templates](./templates.md) for more details.
