use anyhow::Result;
use clap::Args;

use crate::adr::{list_adrs, read_adr_dir_file};

#[derive(Debug, Args)]
pub(crate) struct ListArgs {}

pub(crate) fn run(_args: &ListArgs) -> Result<()> {
    let adr_dir = read_adr_dir_file()?;

    let adrs = list_adrs(&adr_dir)?;
    for adr in adrs {
        println!("{}", adr.display());
    }
    Ok(())
}
