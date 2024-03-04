use std::{
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
};

use crate::adr::{self, get_title, list_adrs};
use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum GenerateCommands {
    Toc {},
    Graph {},
}

pub(crate) fn run(args: &GenerateCommands) -> Result<()> {
    match args {
        GenerateCommands::Toc {} => run_toc(),
        GenerateCommands::Graph {} => run_graph(),
    }
}

fn run_toc() -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;
    let entries = read_dir(adr_dir)?;
    println! {"# Architecture Decision Records\n"};
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let title = get_title(path.as_path())?;
        let file_name = path.file_name().unwrap().to_str().unwrap();

        println!("* [{}]({})", title, file_name);
    }
    Ok(())
}

fn run_graph() -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;
    let adrs = list_adrs(Path::new(&adr_dir))?;

    let items = adrs
        .into_iter()
        .map(|path| {
            let title = get_title(path.as_path()).unwrap();
            let filename = path.file_name().unwrap().to_str().unwrap().to_owned();
            let number = filename.split('-').next().unwrap().parse::<i32>().unwrap();
            (number, title, filename)
        })
        .collect::<Vec<_>>();

    println!("digraph {{\n  node [shape=plaintext]\n  subgraph {{");
    for (number, title, filename) in items {
        let mut path = PathBuf::from(&filename);
        path.set_extension("html");

        println!(
            "\t_{} [label=\"{}\"; URL=\"{}\"];",
            number,
            title,
            &path.display()
        );

        if number > 1 {
            println!(
                "\t_{} -> _{} [style=\"dotted\", weight=1];",
                number - 1,
                number
            );
        }
    }
    println!("  }}\n}}");
    Ok(())
}
