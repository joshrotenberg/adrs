use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::adr::{find_adr_dir, get_links, get_title, list_adrs, read_adr_dir_file};
use anyhow::Result;
use clap::{Args, Subcommand};

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

#[derive(Debug, Args)]
pub(crate) struct GraphArgs {
    /// Link extension
    #[clap(long, short, default_value = "html")]
    extension: String,
    /// Link prefix
    #[clap(long, short)]
    prefix: Option<String>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum GenerateCommands {
    Toc(TocArgs),
    Graph(GraphArgs),
}

pub(crate) fn run(args: &GenerateCommands) -> Result<()> {
    match args {
        GenerateCommands::Toc(args) => run_toc(args),
        GenerateCommands::Graph(args) => run_graph(args),
    }
}

fn run_toc(args: &TocArgs) -> Result<()> {
    let adr_dir = find_adr_dir()?;
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

fn run_graph(args: &GraphArgs) -> Result<()> {
    let adr_dir = read_adr_dir_file()?;
    let adrs = list_adrs(Path::new(&adr_dir))?;

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
        path.set_extension(args.extension.as_str());

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
