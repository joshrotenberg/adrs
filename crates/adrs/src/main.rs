//! adrs - Architecture Decision Records CLI tool.

use adrs_core::{ConfigSource, discover};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "adrs")]
#[command(author, version, about = "Manage Architecture Decision Records", long_about = None)]
struct Cli {
    /// Use next-gen mode (YAML frontmatter, enhanced features)
    #[arg(long, global = true)]
    ng: bool,

    /// Working directory (defaults to current directory)
    #[arg(short = 'C', long = "cwd", global = true)]
    working_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new ADR repository
    Init {
        /// Directory to store ADRs
        #[arg(default_value = "doc/adr")]
        directory: PathBuf,
    },

    /// Create a new ADR
    New {
        /// Title of the ADR
        title: String,

        /// ADR number(s) this supersedes
        #[arg(short, long)]
        supersedes: Option<u32>,

        /// Link to another ADR (format: "TARGET:KIND:REVERSE_KIND")
        #[arg(short, long)]
        link: Option<String>,

        /// Template format to use [default: nygard]
        #[arg(short, long, value_name = "FORMAT")]
        format: Option<String>,

        /// Template variant [default: full]
        #[arg(short, long, value_name = "VARIANT")]
        variant: Option<String>,

        /// Initial status [default: Proposed]
        #[arg(long)]
        status: Option<String>,
    },

    /// Edit an existing ADR
    Edit {
        /// ADR number or title to edit
        adr: String,
    },

    /// List all ADRs
    List,

    /// Link two ADRs together
    Link {
        /// Source ADR number
        source: u32,

        /// Link description (e.g., "Amends")
        link: String,

        /// Target ADR number
        target: u32,

        /// Reverse link description (e.g., "Amended by")
        reverse_link: String,
    },

    /// Show configuration
    Config,

    /// Check repository health
    Doctor,

    /// Generate documentation
    Generate {
        #[command(subcommand)]
        command: GenerateCommands,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate a table of contents
    Toc {
        /// Use ordered list
        #[arg(short, long)]
        ordered: bool,

        /// Intro file to prepend
        #[arg(short, long)]
        intro: Option<PathBuf>,

        /// Outro file to append
        #[arg(short = 'O', long)]
        outro: Option<PathBuf>,

        /// Link prefix
        #[arg(short, long)]
        prefix: Option<String>,
    },

    /// Generate a Graphviz graph
    Graph {
        /// Link prefix for URLs
        #[arg(short, long)]
        prefix: Option<String>,

        /// File extension (default: md)
        #[arg(short, long, default_value = "md")]
        extension: String,
    },

    /// Generate an mdbook
    Book {
        /// Output directory
        #[arg(short, long, default_value = "book")]
        output: PathBuf,

        /// Book title
        #[arg(short, long)]
        title: Option<String>,

        /// Book description
        #[arg(short, long)]
        description: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // For init command, use working_dir or current directory directly
    // For all other commands, discover the project root
    let start_dir = cli
        .working_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    match cli.command {
        Commands::Init { directory } => {
            // Init uses the specified directory directly, no discovery
            commands::init(&start_dir, directory, cli.ng)
        }
        Commands::New {
            title,
            supersedes,
            link,
            format,
            variant,
            status,
        } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::new(
                &discovered.root,
                cli.ng,
                title,
                supersedes,
                link,
                format,
                variant,
                status,
            )
        }
        Commands::Edit { adr } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::edit(&discovered.root, &adr)
        }
        Commands::List => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::list(&discovered.root)
        }
        Commands::Link {
            source,
            link,
            target,
            reverse_link,
        } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::link(&discovered.root, source, &link, target, &reverse_link)
        }
        Commands::Config => {
            let discovered = discover(&start_dir).ok();
            commands::config_with_discovery(&start_dir, discovered)
        }
        Commands::Doctor => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::doctor(&discovered.root)
        }
        Commands::Generate { command } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            match command {
                GenerateCommands::Toc {
                    ordered,
                    intro,
                    outro,
                    prefix,
                } => commands::generate_toc(&discovered.root, ordered, intro, outro, prefix),
                GenerateCommands::Graph { prefix, extension } => {
                    commands::generate_graph(&discovered.root, prefix, &extension)
                }
                GenerateCommands::Book {
                    output,
                    title,
                    description,
                } => commands::generate_book(&discovered.root, &output, title, description),
            }
        }
    }
}

/// Discover config or return a helpful error.
fn discover_or_error(
    start_dir: &std::path::Path,
    explicit_dir: bool,
) -> Result<adrs_core::DiscoveredConfig> {
    let discovered = discover(start_dir).context(if explicit_dir {
        "No ADR repository found in the specified directory. Run 'adrs init' first."
    } else {
        "No ADR repository found. Run 'adrs init' to create one, or use '-C' to specify a directory."
    })?;

    // If using defaults and no project found, that's an error for most commands
    if matches!(discovered.source, ConfigSource::Default)
        && !start_dir.join("doc/adr").exists()
        && !explicit_dir
    {
        anyhow::bail!(
            "No ADR repository found. Run 'adrs init' to create one, or use '-C' to specify a directory."
        );
    }

    Ok(discovered)
}
