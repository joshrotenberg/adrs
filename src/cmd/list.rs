use std::fs::read_to_string;

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct ListArgs {}

pub(crate) fn run(_args: &ListArgs) -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;
    let entries = std::fs::read_dir(adr_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        println!("{}", path.display());
    }
    Ok(())
}
