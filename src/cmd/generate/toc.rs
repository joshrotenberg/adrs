use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;

use crate::adr::{find_adr_dir, get_title, list_adrs};

#[derive(Debug, Args)]
pub(crate) struct TocArgs {
    /// Precede the table of contents with the given intro text
    #[clap(long, short)]
    intro: Option<PathBuf>,
    /// Follow the table of contents with the given outro text
    #[clap(long, short)]
    outro: Option<PathBuf>,
    /// Prefix each decision file link with the given string
    #[clap(long, short)]
    prefix: Option<String>,
}

pub fn run_toc(args: &TocArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;
    let adrs = list_adrs(Path::new(&adr_dir))?;

    println! {"# Architecture Decision Records\n"};
    if let Some(intro) = &args.intro {
        println!("{}", read_to_string(intro)?);
    }
    for path in adrs {
        let title = get_title(&path)?;
        let mut path = PathBuf::from(&path.file_name().unwrap().to_str().unwrap().to_owned());

        path = match &args.prefix {
            Some(prefix) => PathBuf::from(prefix).join(path),
            None => path,
        };

        println!("* [{}]({})", title, &path.display());
    }
    if let Some(outro) = &args.outro {
        println!("\n{}", read_to_string(outro)?);
    }
    Ok(())
}
