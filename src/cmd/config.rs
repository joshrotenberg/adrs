use anyhow::Result;
use clap::Args;

use crate::adr::read_adr_dir_file;

#[derive(Debug, Args)]
pub(crate) struct ConfigArgs {}

pub(crate) fn run(_args: &ConfigArgs) -> Result<()> {
    println!(
        "adrs_bin_dir={}",
        std::env::current_exe().unwrap().parent().unwrap().display()
    );
    println!("adrs_template_dir=embedded");
    if let Ok(adr_dir) = read_adr_dir_file() {
        println!("adrs_dir={}", adr_dir.display());
        return Ok(());
    }
    Ok(())
}
