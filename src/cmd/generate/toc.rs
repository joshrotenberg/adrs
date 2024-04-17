use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;
use regex::Regex;

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
    /// Generate an ordered list with numbered ADR titles
    #[clap(long, short = 'O', default_value_t = false)]
    ordered: bool,
}

pub fn get_ordinal(title: &String) -> Result<(u32, String)> {
    let re = Regex::new(r"^(?<ordinal>\d{1,9})[.)]\s*(?<text>.+$)").unwrap();
    match re.captures(title) {
        Some(caps) => Ok((
            caps["ordinal"].parse::<u32>().unwrap(),
            caps["text"].to_string(),
        )),
        None => Err(anyhow::anyhow!(
            "No ordered list marker found in title '{}'",
            title
        )),
    }
}

pub fn print_ordered_toc(mut toc_lines: Vec<(u32, String, PathBuf)>) -> Result<()> {
    toc_lines.sort_by(|a, b| a.0.cmp(&b.0));
    let mut expected_next_ordinal = 1;
    for line in toc_lines {
        if line.0 != expected_next_ordinal {
            return Err(anyhow::anyhow!(
                "ADR ordering must start at 1 and increase linearly with no gaps"
            ));
        }
        expected_next_ordinal += 1;
        println!("1. [{}]({})", line.1, line.2.display());
    }
    Ok(())
}

pub fn run_toc(args: &TocArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;
    let adrs = list_adrs(Path::new(&adr_dir))?;

    println! {"# Architecture Decision Records\n"};
    if let Some(intro) = &args.intro {
        println!("{}", read_to_string(intro)?);
    }

    let mut toc_lines = Vec::<(u32, String, PathBuf)>::new();
    for path in adrs {
        let title = get_title(&path)?;
        let mut path = PathBuf::from(&path.file_name().unwrap().to_str().unwrap().to_owned());

        path = match &args.prefix {
            Some(prefix) => PathBuf::from(prefix).join(path),
            None => path,
        };

        if !args.ordered {
            println!("* [{}]({})", title, &path.display());
        } else {
            let (ordinal, text) = get_ordinal(&title).unwrap();
            toc_lines.push((ordinal, text, path));
        }
    }
    if args.ordered {
        print_ordered_toc(toc_lines).unwrap();
    }

    if let Some(outro) = &args.outro {
        println!("\n{}", read_to_string(outro)?);
    }
    Ok(())
}
