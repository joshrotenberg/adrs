# {number}. {title}

Date: {date}

## Status

Accepted
{{ for adr in superceded }}
Supercedes [{adr.0}]({adr.1})
{{- endfor }}

{{- for adr in linked }}
{ adr.0 } [{adr.1}]({adr.2})
{{- endfor }}

## Context

The issue motivating this decision, and any context that influences or constrains the decision.

## Decision

The change that we're proposing or have agreed to implement.

## Consequences

What becomes easier or more difficult to do and any risks introduced by the change that will need to be mitigated.
