use anyhow::Result;
use clap::Subcommand;

pub mod book;
pub mod graph;
pub mod toc;

#[derive(Debug, Subcommand)]
pub(crate) enum GenerateCommands {
    /// Generate a table of contents
    Toc(toc::TocArgs),
    /// Generate a graph of the ADRs
    Graph(graph::GraphArgs),
    /// Generate a book of the ADRs
    Book(book::BookArgs),
}

pub(crate) fn run(args: &GenerateCommands) -> Result<()> {
    match args {
        GenerateCommands::Toc(args) => toc::run_toc(args),
        GenerateCommands::Graph(args) => graph::run_graph(args),
        GenerateCommands::Book(args) => book::run_book(args),
    }
}
