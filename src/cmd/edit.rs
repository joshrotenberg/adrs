use std::{fs::read_to_string, path::Path};

use anyhow::{Context, Result};
use clap::Args;
use edit::edit;

use crate::adr::{find_adr, find_adr_dir};

#[derive(Debug, Args)]
pub(crate) struct EditArgs {
    /// The number of the ADR to edit
    name: String,
}

pub(crate) fn run(args: &EditArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;

    let adr = find_adr(Path::new(&adr_dir), &args.name)?;
    let content = read_to_string(adr.clone())?;
    let edited = edit(content)?;

    std::fs::write(adr.as_path(), edited)?;

    Ok(())
}
