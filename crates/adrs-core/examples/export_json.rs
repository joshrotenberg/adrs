//! Export a repository to the JSON-ADR format.
//!
//! JSON-ADR is a machine-readable representation of a decision log -- handy for
//! feeding ADRs to other tools or to AI agents that reason over them.
//!
//! Run with: `cargo run -p adrs-core --example export_json`

use adrs_core::{Repository, export_repository};

fn main() -> adrs_core::Result<()> {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let repo = Repository::init(tmp.path(), None, false)?;
    repo.new_adr("Use gRPC for service-to-service calls")?;

    let export = export_repository(&repo)?;
    let json = serde_json::to_string_pretty(&export).expect("serialize JSON-ADR");
    println!("{json}");

    Ok(())
}
