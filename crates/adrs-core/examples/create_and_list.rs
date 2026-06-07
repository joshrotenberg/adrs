//! Create an ADR repository, add a decision, and list all records.
//!
//! Run with: `cargo run -p adrs-core --example create_and_list`

use adrs_core::Repository;

fn main() -> adrs_core::Result<()> {
    // Use a throwaway directory so the example is self-contained.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let repo = Repository::init(tmp.path(), None, false)?;

    // `init` seeds ADR 0001 ("Record architecture decisions").
    let (adr, path) = repo.new_adr("Use PostgreSQL for persistence")?;
    println!("Created ADR {:04} at {}", adr.number, path.display());

    println!("\nAll ADRs:");
    for adr in repo.list()? {
        println!("  {:04}  {}  [{}]", adr.number, adr.title, adr.status);
    }

    Ok(())
}
