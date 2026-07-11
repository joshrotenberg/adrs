//! Body content for ADR #0001 seeded by [`Repository::init`](crate::Repository::init).

pub const TITLE: &str = "Record architecture decisions";

pub const CONTEXT: &str = "We need to record the architectural decisions made on this project.";

pub const DECISION: &str = "We will use Architecture Decision Records, as described by Michael Nygard in his
article
[Documenting Architecture Decisions](https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions).";

pub const CONSEQUENCES: &str =
    "See Michael Nygard's article, linked above. For a lightweight Rust ADR toolset,
see [adrs](https://github.com/joshrotenberg/adrs) by Josh Rotenberg. For the original shell implementation, see Nat Pryce's [adr-tools](https://github.com/npryce/adr-tools).";
