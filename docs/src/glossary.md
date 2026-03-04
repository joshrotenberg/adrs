# Glossary

Key terms and concepts used throughout the `adrs` documentation.

## A

### ADR (Architecture Decision Record)

A short document that captures a single architectural decision. ADRs record the context, decision, and consequences of significant technical choices. See [ADRs](./adrs/README.md).

### adr-tools

The original shell-based ADR tooling created by Nat Pryce. `adrs` is designed as a drop-in replacement with additional features. See [adr-tools on GitHub](https://github.com/npryce/adr-tools).

## C

### Compatible Mode

The default operating mode of `adrs` that maintains full compatibility with adr-tools. Uses `.adr-dir` configuration and plain markdown without frontmatter. See [Compatible Mode](./modes/compatible.md).

## F

### Format

The overall template structure for ADRs. `adrs` supports two formats:

- **Nygard**: Classic adr-tools format
- **MADR**: Markdown Architectural Decision Records 4.0.0

See [Templates](./templates/README.md).

### Frontmatter

YAML metadata at the beginning of a markdown file, delimited by `---` markers. Used in NextGen mode to store structured data like status, date, and links. See [Frontmatter](./frontmatter.md).

## L

### Link Kind

The type of relationship between two ADRs:

- **supersedes/superseded-by**: One ADR replaces another
- **amends/amended-by**: One ADR modifies another
- **related**: General relationship

## M

### MADR (Markdown Architectural Decision Records)

A structured ADR format with YAML frontmatter and additional sections for decision drivers, options comparison, and stakeholder tracking. Version 4.0.0 is supported. See [MADR Format](./templates/madr.md).

## N

### NextGen Mode

An enhanced operating mode that enables YAML frontmatter, structured metadata, and full MADR support. Enabled with `--ng` flag or `adrs.toml` configuration. See [NextGen Mode](./modes/nextgen.md).

### Nygard Format

The classic ADR format created by Michael Nygard, consisting of Title, Date, Status, Context, Decision, and Consequences sections. See [Nygard Format](./templates/nygard.md).

## S

### Status

The current state of an ADR:

| Status | Description |
|--------|-------------|
| Proposed | Under discussion |
| Accepted | Approved and in effect |
| Deprecated | No longer recommended |
| Superseded | Replaced by another ADR |

### Supersedes/Superseded

A relationship where one ADR replaces another. The newer ADR "supersedes" the older one, and the older one is "superseded by" the newer one.

## T

### Template

A file that defines the structure of new ADRs. Templates use Jinja2 syntax for variable substitution. See [Templates](./templates/README.md).

## V

### Variant

The level of detail in a template:

| Variant | Description |
|---------|-------------|
| `full` | All sections with guidance comments |
| `minimal` | Essential sections only |
| `bare` | All sections, no guidance |
| `bare-minimal` | Core sections only, empty |

See [Variants](./templates/variants.md).

## Related Documentation

- [Installation](./installation.md)
- [Configuration](./configuration.md)
- [Commands](./commands/README.md)
