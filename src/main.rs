use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{fs::create_dir_all, path::PathBuf};

use crate::adr::AdrBuilder;

mod adr;

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
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes the directory of architecture decision records
    Init {
        /// Directory to initialize
        #[arg(default_value = "doc/adr")]
        directory: PathBuf,
    },
    /// Create a new, numbered ADR
    New {
        /// A reference to a previous decision to supercede with this new one
        #[arg(short, long)]
        superceded: Option<Vec<String>>,
        /// Link the new ADR to a previous ADR
        #[arg(short, long)]
        link: Option<Vec<String>>,
        /// Title of the new ADR
        #[arg(trailing_var_arg = true, required = true)]
        title: Vec<String>,
    },
    /// Link ADRs
    Link {
        /// The source ADR number or file name match
        source: i32,
        /// Description of the link to create in the source ADR
        link: String,
        /// The target ADR number or file name match
        target: i32,
        /// Description of the link to create in the target ADR
        reverse_link: String,
    },
    /// List ADRs
    List {
        /// Directory to list
        #[arg(default_value = "doc/adr")]
        directory: PathBuf,
    },
    /// Show the current configuration
    Config {},
    /// Generates summary documentation about the ADRs
    #[command(subcommand)]
    Generate(GenerateCommands),
}

#[derive(Debug, Subcommand)]
enum GenerateCommands {
    Toc {},
    Graph {},
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { directory } => {
            create_dir_all(directory)?;
            let adr = AdrBuilder::new()
                .title("Record architecture decisions")
                .status("Accepted")
                .context("We need to record the architectural decisions made on this project.")
                .decision("We will use ADRs to record the decisions made on this project.")
                .consequences("We will have a record of the decisions made on this project.")
                .write(directory)?;

            tracing::debug!("Created {}", adr);
            std::fs::write(
                std::env::current_dir()?.join(".adr-dir"),
                directory.to_str().unwrap(),
            )?;
            tracing::debug!("Wrote .adr-dir");
        }
        Commands::New {
            superceded,
            link,
            title,
        } => {
            tracing::debug!(?title);
            tracing::debug!(?superceded);
            tracing::debug!(?link);
        }
        Commands::Link {
            source,
            link,
            target,
            reverse_link,
        } => {
            tracing::debug!(?source);
            tracing::debug!(?link);
            tracing::debug!(?target);
            tracing::debug!(?reverse_link);
        }
        Commands::List { directory } => {
            tracing::debug!("list");
            let entries = std::fs::read_dir(directory)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                tracing::debug!(?path);
            }
        }
        Commands::Config {} => {
            tracing::debug!("config");
        }
        Commands::Generate(c) => match c {
            GenerateCommands::Toc {} => {
                tracing::debug!("generate toc");
            }
            GenerateCommands::Graph {} => {
                tracing::debug!("generate graph");
            }
        },
    }
    Ok(())
}
