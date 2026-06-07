//! Create two ADRs and connect them with a typed, bidirectional link.
//!
//! Run with: `cargo run -p adrs-core --example link_adrs`

use adrs_core::{LinkKind, Repository};

fn main() -> adrs_core::Result<()> {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let repo = Repository::init(tmp.path(), None, false)?;

    let (amending, _) = repo.new_adr("Adopt event sourcing")?;
    let (amended, _) = repo.new_adr("Use a single relational database")?;

    // `amending` amends `amended`. The reverse ("Amended by") link is written
    // to the target automatically.
    repo.link(
        amending.number,
        amended.number,
        LinkKind::Amends,
        LinkKind::AmendedBy,
    )?;

    for adr in repo.list()? {
        println!("{:04}  {}", adr.number, adr.title);
        for link in &adr.links {
            println!("    {:?} -> {:04}", link.kind, link.target);
        }
    }

    Ok(())
}
