# template

Manage ADR templates.

## Usage

```
adrs template <COMMAND>
```

## Subcommands

| Command | Description |
|---------|-------------|
| `list` | List available templates |
| `show` | Show a template's content |

## list

List all available template formats and variants.

```sh
adrs template list
```

Output:

```
Available templates:

nygard (default)
  Variants: full (default), minimal, bare

madr
  Variants: full (default), minimal, bare
```

## show

Display the content of a specific template.

```sh
adrs template show <FORMAT> [--variant <VARIANT>]
```

### Options

| Option | Description |
|--------|-------------|
| `-v, --variant <VARIANT>` | Template variant: full, minimal, bare |

### Examples

```sh
# Show default Nygard template (full variant)
adrs template show nygard

# Show minimal MADR template
adrs template show madr --variant minimal

# Show bare template
adrs template show nygard --variant bare
```

## Template Formats

### Nygard (Classic)

The original ADR format by Michael Nygard with sections:
- Status
- Context
- Decision
- Consequences

### MADR 4.0.0

Markdown Any Decision Record format with additional sections:
- Status
- Deciders
- Date
- Context and Problem Statement
- Decision Drivers
- Considered Options
- Decision Outcome
- Consequences

## Template Variants

| Variant | Description |
|---------|-------------|
| `full` | Complete template with all sections and guidance |
| `minimal` | Essential sections only |
| `bare` | Minimal structure, just headings |

## See Also

- [new](./new.md) - Create ADRs using templates
- [Templates](../templates.md) - Template customization guide
