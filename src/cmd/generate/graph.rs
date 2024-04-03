use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::adr::{find_adr_dir, get_links, get_title, list_adrs};

#[derive(Debug, Args)]
pub(crate) struct GraphArgs {
    /// Link extension
    #[clap(long, short, default_value = "html")]
    extension: String,
    /// Link prefix
    #[clap(long, short)]
    prefix: Option<String>,
}

pub fn run_graph(args: &GraphArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;
    let adrs = list_adrs(Path::new(&adr_dir))?;

    let extension = args
        .extension
        .trim_start_matches(|c| char::is_ascii_punctuation(&c));
    let items = adrs
        .into_iter()
        .map(|path| {
            let title = get_title(path.as_path()).unwrap();
            let filename = path.file_name().unwrap().to_str().unwrap().to_owned();
            let number = filename.split('-').next().unwrap().parse::<i32>().unwrap();
            let links = get_links(path.as_path()).unwrap();
            (number, title, filename, links)
        })
        .collect::<Vec<_>>();

    println!("digraph {{\n  node [shape=plaintext]\n  subgraph {{");
    for (number, title, filename, _links) in &items {
        let mut path = PathBuf::from(&filename);
        path.set_extension(extension);

        path = match &args.prefix {
            Some(prefix) => PathBuf::from(prefix).join(path),
            None => path,
        };

        println!(
            "\t_{} [label=\"{}\"; URL=\"{}\"];",
            number,
            title,
            &path.display()
        );

        if *number > 1 {
            println!(
                "\t_{} -> _{} [style=\"dotted\", weight=1];",
                number - 1,
                number
            );
        }
    }
    println!("  }}");
    for (number, _title, _filename, links) in &items {
        for (link, title, _file) in links {
            let linked_number = title.split_once(". ").unwrap().0;
            println!(
                "  _{} -> _{} [label=\"{}\", weight=0];",
                number, linked_number, link
            )
        }
    }
    println!("}}");
    Ok(())
}
