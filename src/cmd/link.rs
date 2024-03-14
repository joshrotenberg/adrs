use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;

use crate::adr::{append_status, find_adr, find_adr_dir, get_title};

#[derive(Debug, Args)]
pub(crate) struct LinkArgs {
    /// The source Architectural Decision Record number or file name match
    source: String,
    /// Description of the link to create in the source Architectural Decision Record
    link: String,
    /// The target Architectural Decision Record number or file name match
    target: i32,
    /// Description of the link to create in the target Architectural Decision Record
    reverse_link: String,
}

pub(crate) fn run(args: &LinkArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;

    let source =
        find_adr(Path::new(&adr_dir), &args.source).context("Unable to find source ADR")?;
    let source_filename = source.file_name().unwrap().to_str().unwrap();
    let source_title = get_title(&source).context("Unable to get title for source ADR")?;

    let target = find_adr(Path::new(&adr_dir), &args.target.to_string())
        .context("Unable to find target ADR")?;
    let target_filename = target.file_name().unwrap().to_str().unwrap();
    let target_title = get_title(&target).context("Unable to get title for target ADR")?;

    let source_link = format!("{} [{}]({})", args.link, target_title, target_filename);
    let target_link = format!(
        "{} [{}]({})",
        args.reverse_link, source_title, source_filename
    );

    append_status(&source, &source_link).context("Unable to append status for source ADR")?;
    append_status(&target, &target_link).context("Unable to append status for target ADR")?;

    Ok(())
}
