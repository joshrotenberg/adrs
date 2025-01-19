use std::path::PathBuf;

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

    // find the template directory. if ADRS_TEMPLATE is set, use that. otherwise, check
    // to see if there is an adr dir, and if it has a templates directory, use that.
    // otherwise use embedded
    if let Ok(template_file) = std::env::var("ADRS_TEMPLATE") {
        let mut path = PathBuf::from(template_file);
        path.pop();
        println!("adrs_template_dir={}", path.display());
    } else if let Ok(adr_dir) = read_adr_dir_file() {
        if adr_dir.join("templates").exists() {
            println!("adrs_template_dir={}", adr_dir.join("templates").display());
        } else {
            println!("adrs_template_dir=embedded");
        }
    } else {
        println!("adrs_template_dir=embedded");
    }

    if let Ok(adr_dir) = read_adr_dir_file() {
        println!("adrs_dir={}", adr_dir.display());
    }
    Ok(())
}
