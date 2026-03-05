# Templates

<!-- toc -->

The template system uses [minijinja](https://github.com/mitsuhiko/minijinja) (Jinja2 syntax) to generate ADR content.

## Basic Usage

```rust
use adrs_core::{TemplateEngine, TemplateFormat, TemplateVariant, Adr};

let engine = TemplateEngine::new()
    .with_format(TemplateFormat::Madr)
    .with_variant(TemplateVariant::Minimal);

let adr = Adr::new(1, "Use PostgreSQL");
let content = engine.render(&adr)?;
```

## Formats

| Format | Description |
|--------|-------------|
| `Nygard` | Classic ADR format (Context, Decision, Consequences) |
| `Madr` | MADR 4.0.0 format (more structured, includes decision drivers) |

```rust
use adrs_core::TemplateFormat;

let format = TemplateFormat::Nygard;  // Default
let format = TemplateFormat::Madr;
```

## Variants

| Variant | Description |
|---------|-------------|
| `Full` | All sections with guidance comments |
| `Minimal` | Essential sections only |
| `Bare` | All sections, no guidance |
| `BareMinimal` | Core sections only, empty |

```rust
use adrs_core::TemplateVariant;

let variant = TemplateVariant::Full;      // Default
let variant = TemplateVariant::Minimal;
let variant = TemplateVariant::Bare;
let variant = TemplateVariant::BareMinimal;
```

## TemplateEngine

### Builder Pattern

```rust
use adrs_core::{TemplateEngine, TemplateFormat, TemplateVariant};

let engine = TemplateEngine::new()
    .with_format(TemplateFormat::Madr)
    .with_variant(TemplateVariant::Minimal);
```

### Custom Templates

```rust
use adrs_core::TemplateEngine;

// Load from file
let engine = TemplateEngine::from_file("templates/custom.md")?;

// Load from string
let template = "# {{ number }}. {{ title }}\n\n...";
let engine = TemplateEngine::from_string(template)?;
```

## Template Variables

Available variables in templates:

| Variable | Type | Description |
|----------|------|-------------|
| `number` | `u32` | ADR number |
| `title` | `String` | Decision title |
| `date` | `String` | Formatted date |
| `status` | `String` | Current status |
| `decision_makers` | `Vec<String>` | Who made the decision |
| `consulted` | `Vec<String>` | Who was consulted |
| `informed` | `Vec<String>` | Who was informed |
| `context` | `String` | Context section |
| `decision` | `String` | Decision section |
| `consequences` | `String` | Consequences section |

## Example: Custom Template

```jinja2
---
number: {{ number }}
title: {{ title }}
date: {{ date }}
status: {{ status }}
{% if tags %}tags:
{% for tag in tags %}  - {{ tag }}
{% endfor %}{% endif %}
---

# {{ number }}. {{ title }}

## Context

{{ context | default("Describe the context...") }}

## Decision

{{ decision | default("Describe the decision...") }}

## Consequences

{{ consequences | default("Describe the consequences...") }}
```

## See Also

- [Template Reference](../../reference/templates/README.md) - Built-in templates
- [ADR-0007: Use minijinja](../../reference/adrs/0007-use-minijinja-for-templates.md)
