# 7. Use minijinja for templates

Date: 2025-01-21

## Status

Accepted

## Context

ADR tools need to generate markdown files from templates. The template system should:

- Support both built-in templates (Nygard, MADR) and custom user templates
- Be familiar to users (similar syntax to popular template engines)
- Be lightweight and fast
- Support conditional logic for mode-specific output (compatible vs nextgen)
- Have good error messages for template debugging

The original implementation used handlebars-rust. Other options include:

- **Tera**: Full-featured, Django/Jinja2-like syntax
- **minijinja**: Lightweight Jinja2 implementation, same author as Tera
- **askama**: Compile-time templates, type-safe but less flexible
- **handlebars-rust**: Mustache-like, used in v1

## Decision

Use minijinja for template rendering because:

- Jinja2 syntax is widely known (Python, Ansible, Hugo, etc.)
- Minimal dependencies and fast compilation
- Excellent error messages with source locations
- Supports all features we need (conditionals, loops, filters)
- Active maintenance by the Tera author (Armin Ronacher)
- Easy to embed custom templates from files

Built-in templates provided:
- **Nygard**: Michael Nygard's original ADR format (default)
- **MADR**: Markdown Any Decision Records format

## Consequences

- Users familiar with Jinja2/Django templates can easily create custom templates
- Template syntax is consistent with many other tools
- Slightly different from handlebars used in v1 (migration needed for custom templates)
- Templates can conditionally include frontmatter based on mode
- Good debugging experience when templates have errors
