//! adrs - Architecture Decision Records CLI tool.

use adrs_core::{ConfigSource, discover};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "adrs")]
#[command(author, version)]
#[command(about = "Manage Architecture Decision Records")]
#[command(long_about = "\
A command-line tool for creating and managing Architecture Decision Records (ADRs).

Compatible with adr-tools repositories. Supports both Nygard and MADR 4.0.0 formats.

GETTING STARTED:
  adrs init                    Create a new ADR repository
  adrs new \"My Decision\"       Create your first ADR
  adrs list                    View all ADRs
  adrs doctor                  Check repository health

FORMATS:
  nygard    Classic adr-tools format (default)
  madr      MADR 4.0.0 with structured metadata

EXAMPLES:
  adrs init                                      Initialize repository
  adrs new --format madr \"Use PostgreSQL\"       Create MADR-format ADR
  adrs new --supersedes 2 \"Use MySQL instead\"   Supersede an ADR
  adrs link 3 Amends 1                          Link two ADRs (auto-derives reverse)
  adrs generate toc > doc/adr/README.md         Generate table of contents

DOCUMENTATION: https://joshrotenberg.github.io/adrs-book/")]
struct Cli {
    /// Enable NextGen mode with YAML frontmatter
    #[arg(long, global = true)]
    ng: bool,

    /// Run from a different directory
    #[arg(short = 'C', long = "cwd", global = true, value_name = "DIR")]
    working_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new ADR repository
    Init {
        /// Directory to store ADRs [default: doc/adr]
        #[arg(default_value = "doc/adr")]
        directory: PathBuf,
    },

    /// Create a new ADR
    New {
        /// Title of the ADR
        title: String,

        /// Supersede an existing ADR by number
        #[arg(short, long, value_name = "NUMBER")]
        supersedes: Option<u32>,

        /// Link to another ADR (format: "TARGET:KIND:REVERSE_KIND")
        #[arg(short, long, value_name = "LINK")]
        link: Option<String>,

        /// Template format: nygard, madr [default: nygard]
        #[arg(short, long, value_name = "FORMAT")]
        format: Option<String>,

        /// Template variant: full, minimal, bare [default: full]
        #[arg(short, long, value_name = "VARIANT")]
        variant: Option<String>,

        /// Initial status [default: Proposed]
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,

        /// Tags for categorization (comma-separated, requires --ng)
        #[arg(short = 't', long, value_name = "TAGS", value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// Edit an existing ADR
    Edit {
        /// ADR number or title (supports fuzzy matching)
        adr: String,
    },

    /// List all ADRs
    List {
        /// Filter by status (e.g., proposed, accepted, deprecated, superseded)
        #[arg(short, long, value_name = "STATUS")]
        status: Option<String>,

        /// Filter by date (show ADRs from this date onwards, YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        since: Option<String>,

        /// Filter by date (show ADRs up to this date, YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        until: Option<String>,

        /// Filter by decision maker (MADR format)
        #[arg(long, value_name = "NAME")]
        decider: Option<String>,

        /// Filter by tag
        #[arg(short = 't', long, value_name = "TAG")]
        tag: Option<String>,

        /// Show detailed output (number, status, date, title)
        #[arg(short = 'l', long)]
        long: bool,
    },

    /// Search ADRs for matching content
    Search {
        /// Search query
        query: String,

        /// Search titles only
        #[arg(short = 't', long)]
        title: bool,

        /// Filter by status
        #[arg(short, long, value_name = "STATUS")]
        status: Option<String>,

        /// Case-sensitive search
        #[arg(short = 'c', long)]
        case_sensitive: bool,
    },

    /// Link two ADRs together
    Link {
        /// Source ADR number
        source: u32,

        /// Link description (e.g., "Amends", "Supersedes", "Relates to")
        link: String,

        /// Target ADR number
        target: u32,

        /// Reverse link description (auto-derived if omitted)
        reverse_link: Option<String>,
    },

    /// Change an ADR's status
    Status {
        /// ADR number
        adr: u32,

        /// New status (proposed, accepted, deprecated, superseded, rejected, or custom)
        status: String,

        /// For 'superseded' status: the ADR number that supersedes this one
        #[arg(long, value_name = "NUMBER")]
        by: Option<u32>,
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

    /// Export ADRs to different formats
    Export {
        #[command(subcommand)]
        command: ExportCommands,
    },

    /// Import ADRs from different formats
    Import {
        #[command(subcommand)]
        command: ImportCommands,
    },

    /// Manage ADR templates
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate a table of contents
    Toc {
        /// Use ordered list (1. 2. 3.)
        #[arg(short, long)]
        ordered: bool,

        /// Prepend content from file
        #[arg(short, long, value_name = "FILE")]
        intro: Option<PathBuf>,

        /// Append content from file
        #[arg(short = 'O', long, value_name = "FILE")]
        outro: Option<PathBuf>,

        /// Prefix for ADR links
        #[arg(short, long, value_name = "PREFIX")]
        prefix: Option<String>,
    },

    /// Generate a Graphviz graph
    Graph {
        /// Prefix for node URLs
        #[arg(short, long, value_name = "PREFIX")]
        prefix: Option<String>,

        /// File extension for links [default: md]
        #[arg(short, long, default_value = "md")]
        extension: String,
    },

    /// Generate an mdbook
    Book {
        /// Output directory [default: book]
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

#[derive(Subcommand)]
enum ExportCommands {
    /// Export ADRs to JSON-ADR format
    Json {
        /// Export a single ADR by number
        #[arg(value_name = "NUMBER")]
        adr: Option<u32>,

        /// Export from a directory (no repository required)
        #[arg(short, long, value_name = "PATH")]
        dir: Option<PathBuf>,

        /// Pretty-print the JSON output
        #[arg(short, long)]
        pretty: bool,

        /// Export metadata only (excludes content, includes source_uri)
        #[arg(long)]
        metadata_only: bool,

        /// Base URL for source_uri (e.g., https://github.com/org/repo/blob/main/doc/adr)
        #[arg(long, value_name = "URL")]
        base_url: Option<String>,
    },
}

#[derive(Subcommand)]
enum ImportCommands {
    /// Import ADRs from JSON-ADR format
    Json {
        /// JSON-ADR file to import (use "-" for stdin)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Import to a directory (no repository required)
        #[arg(short, long, value_name = "PATH")]
        dir: Option<PathBuf>,

        /// Overwrite existing files
        #[arg(short, long)]
        overwrite: bool,

        /// Renumber ADRs starting from next available number (append to existing ADRs)
        #[arg(short, long, alias = "append")]
        renumber: bool,

        /// Preview import without writing files
        #[arg(long)]
        dry_run: bool,

        /// Use next-gen mode with YAML frontmatter
        #[arg(long)]
        ng: bool,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// List available templates
    List,

    /// Show a template's content
    Show {
        /// Template format: nygard, madr
        format: String,

        /// Template variant: full, minimal, bare, bare-minimal
        #[arg(short, long, value_name = "VARIANT")]
        variant: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let start_dir = cli
        .working_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    match cli.command {
        Commands::Init { directory } => commands::init(&start_dir, directory, cli.ng),
        Commands::New {
            title,
            supersedes,
            link,
            format,
            variant,
            status,
            tags,
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
                tags,
            )
        }
        Commands::Edit { adr } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::edit(&discovered.root, &adr)
        }
        Commands::List {
            status,
            since,
            until,
            decider,
            tag,
            long,
        } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::list(&discovered.root, status, since, until, decider, tag, long)
        }
        Commands::Search {
            query,
            title,
            status,
            case_sensitive,
        } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::search(&discovered.root, &query, title, status, case_sensitive)
        }
        Commands::Link {
            source,
            link,
            target,
            reverse_link,
        } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::link(
                &discovered.root,
                source,
                &link,
                target,
                reverse_link.as_deref(),
            )
        }
        Commands::Status { adr, status, by } => {
            let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
            commands::status(&discovered.root, adr, &status, by)
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
        Commands::Export { command } => match command {
            ExportCommands::Json {
                adr,
                dir,
                pretty,
                metadata_only,
                base_url,
            } => {
                if let Some(ref dir_path) = dir {
                    // Export from arbitrary directory - no repo needed
                    commands::export_json(
                        &start_dir,
                        adr,
                        Some(dir_path),
                        pretty,
                        metadata_only,
                        base_url,
                    )
                } else {
                    // Export from repository
                    let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
                    commands::export_json(
                        &discovered.root,
                        adr,
                        None,
                        pretty,
                        metadata_only,
                        base_url,
                    )
                }
            }
        },
        Commands::Import { command } => match command {
            ImportCommands::Json {
                file,
                dir,
                overwrite,
                renumber,
                dry_run,
                ng,
            } => {
                if let Some(ref dir_path) = dir {
                    // Import to arbitrary directory - no repo needed
                    commands::import_json(
                        &start_dir,
                        &file,
                        Some(dir_path),
                        overwrite,
                        renumber,
                        dry_run,
                        ng,
                    )
                } else {
                    // Import to repository
                    let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
                    commands::import_json(
                        &discovered.root,
                        &file,
                        None,
                        overwrite,
                        renumber,
                        dry_run,
                        ng,
                    )
                }
            }
        },
        Commands::Template { command } => match command {
            TemplateCommands::List => commands::template_list(),
            TemplateCommands::Show { format, variant } => {
                commands::template_show(&format, variant.as_deref())
            }
        },
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
