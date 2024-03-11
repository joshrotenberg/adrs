use anyhow::Result;
use clap::Args;

use crate::adr::{find_adr_dir, list_adrs};

#[derive(Debug, Args)]
pub(crate) struct ListArgs {}

pub(crate) fn run(_args: &ListArgs) -> Result<()> {
    let adr_dir = find_adr_dir()?;

    let adrs = list_adrs(&adr_dir)?;
    for adr in adrs {
        println!("{}", adr.display());
    }
    Ok(())
}
