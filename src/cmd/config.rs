use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct ConfigArgs {}

pub(crate) fn run(_args: &ConfigArgs) -> Result<()> {
    tracing::debug!("config");
    println!(
        "adrs_bin_dir={}",
        std::env::current_exe().unwrap().parent().unwrap().display()
    );
    println!("adrs_template_dir=embedded");
    Ok(())
}
