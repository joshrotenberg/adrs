use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;

pub mod adr;
mod cmd;

#[derive(Debug, Serialize)]
struct TemplateContext {
    title: String,
    number: i32,
    date: String,
    status: String,
    context: String,
    decision: String,
    consequences: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None )]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes the directory of Architecture Decision Records
    Init(cmd::init::InitArgs),
    /// Create a new, numbered Architectural Decision Record
    New(cmd::new::NewArgs),
    /// Edit an existing Architectural Decision Record
    Edit(cmd::edit::EditArgs),
    /// Link Architectural Decision Records
    Link(cmd::link::LinkArgs),
    /// List Architectural Decision Records
    List(cmd::list::ListArgs),
    /// Show the current configuration
    Config(cmd::config::ConfigArgs),
    /// Generates summary documentation about the Architectural Decision Records
    #[command(subcommand)]
    Generate(cmd::generate::GenerateCommands),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => {
            cmd::init::run(args)?;
        }
        Commands::New(args) => {
            cmd::new::run(args)?;
        }
        Commands::Edit(args) => {
            cmd::edit::run(args)?;
        }
        Commands::Link(args) => {
            cmd::link::run(args)?;
        }
        Commands::List(args) => {
            cmd::list::run(args)?;
        }
        Commands::Config(args) => {
            cmd::config::run(args)?;
        }
        Commands::Generate(args) => {
            cmd::generate::run(args)?;
        }
    }
    Ok(())
}
