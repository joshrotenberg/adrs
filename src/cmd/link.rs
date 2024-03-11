use std::path::Path;

use anyhow::Result;
use clap::Args;

use crate::adr::{self, append_status, find_adr_dir};

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
    let adr_dir = find_adr_dir()?;

    let source = adr::find_adr(Path::new(&adr_dir), &args.source)?;
    let source_filename = source.file_name().unwrap().to_str().unwrap();
    let source_title = adr::get_title(&source)?;

    let target = adr::find_adr(Path::new(&adr_dir), &args.target.to_string())?;
    let target_filename = target.file_name().unwrap().to_str().unwrap();
    let target_title = adr::get_title(&target)?;

    let source_link = format!("{} [{}]({})", args.link, target_title, target_filename);
    let target_link = format!(
        "{} [{}]({})",
        args.reverse_link, source_title, source_filename
    );

    append_status(&source, &source_link)?;
    append_status(&target, &target_link)?;

    Ok(())
}
