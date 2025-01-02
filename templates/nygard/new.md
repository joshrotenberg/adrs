# {{number}}. {{title}}

Date: {{date}}

## Status

Accepted
{{#each superseded}}
{{this}}
{{/each}}
{{#each linked}}
{{this}}
{{/each}}

## Context

{{#if init}}
We need to record the architectural decisions made on this project.
{{else}}
The issue motivating this decision, and any context that influences or constrains the decision.
{{/if}}

## Decision

{{#if init}}
We will use Architecture Decision Records, as [described by Michael Nygard](http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions).
{{else}}
The change that we're proposing or have agreed to implement.
{{/if}}

## Consequences

{{#if init}}
See Michael Nygard's article, linked above. For a lightweight ADR toolset, see Nat Pryce's [adr-tools](https://github.com/npryce/adr-tools).
{{else}}
What becomes easier or more difficult to do and any risks introduced by the change that will need to be mitigated.
{{/if}}
