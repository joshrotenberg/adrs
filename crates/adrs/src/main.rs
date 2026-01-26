//! adrs - Architecture Decision Records CLI tool.

use adrs_core::{ConfigSource, discover};
use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use std::io;
use std::path::PathBuf;

mod commands;

#[cfg(feature = "mcp")]
mod mcp;

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

MODES:
  Compatible (default)    Works with adr-tools, metadata in markdown
  NextGen (--ng)          YAML frontmatter for richer metadata (tags, custom fields)

EXAMPLES:
  adrs init                                      Initialize repository
  adrs new --format madr \"Use PostgreSQL\"       Create MADR-format ADR
  adrs new --supersedes 2 \"Use MySQL instead\"   Supersede an ADR
  adrs link 3 Amends 1                          Link two ADRs (auto-derives reverse)
  adrs generate toc > doc/adr/README.md         Generate table of contents

DOCUMENTATION: https://joshrotenberg.github.io/adrs-book/")]
struct Cli {
    /// Enable NextGen mode with YAML frontmatter for richer metadata
    #[arg(
        long,
        global = true,
        help = "Enable NextGen mode with YAML frontmatter"
    )]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs init                    Initialize in doc/adr (default)
  adrs init docs/decisions     Use custom directory
  adrs --ng init               Initialize with NextGen mode (YAML frontmatter)

Creates the ADR directory and an initial ADR documenting the use of ADRs.
If ADRs already exist in the directory, they are preserved.")]
    Init {
        /// Directory to store ADRs [default: doc/adr]
        #[arg(default_value = "doc/adr")]
        directory: PathBuf,
    },

    /// Create a new ADR
    #[command(after_long_help = "\
EXAMPLES:
  adrs new \"Use PostgreSQL for persistence\"      Basic ADR
  adrs new --format madr \"Use React\"             MADR format with structured sections
  adrs new --supersedes 2 \"Use MySQL instead\"    Supersede ADR 2
  adrs new --link \"2:Amends:Amended by\" \"...\"    Create with link to ADR 2
  adrs new --status accepted \"Already decided\"   Start with accepted status
  adrs new --no-edit \"Quick note\"                Create without opening editor

NEXTGEN MODE (--ng):
  adrs --ng new -t api,security \"Auth Design\"   Create with tags (requires --ng)
  adrs --ng new \"My Decision\"                   Enable YAML frontmatter metadata

LINK FORMAT:
  The --link option uses format: TARGET:KIND:REVERSE_KIND
  Example: \"2:Amends:Amended by\" links to ADR 2 with bidirectional links")]
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

        /// Create ADR without opening editor (for scripting/CI)
        #[arg(long)]
        no_edit: bool,
    },

    /// Edit an existing ADR
    Edit {
        /// ADR number or title (supports fuzzy matching)
        adr: String,
    },

    /// List all ADRs
    #[command(after_long_help = "\
EXAMPLES:
  adrs list                              List all ADRs (default format)
  adrs list -l                           Detailed view with status and date
  adrs list --status accepted            Show only accepted ADRs
  adrs list --since 2024-01-01           ADRs created since Jan 1, 2024
  adrs list --until 2024-06-30           ADRs created before July 2024
  adrs list --tag security               Filter by tag (requires --ng mode)
  adrs list --decider \"Alice\"            Filter by decision maker (MADR)

COMBINING FILTERS:
  adrs list -l --status accepted --since 2024-01-01
  adrs list --tag api --status proposed")]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs search postgres                   Search all content for 'postgres'
  adrs search -t database                Search titles only
  adrs search --status accepted auth     Search accepted ADRs for 'auth'
  adrs search -c PostgreSQL              Case-sensitive search

TIPS:
  - Search is case-insensitive by default
  - Searches both title and full content unless -t is used
  - Combine with --status to narrow results")]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs link 3 Supersedes 1               ADR 3 supersedes ADR 1
  adrs link 5 Amends 2                   ADR 5 amends ADR 2
  adrs link 4 \"Relates to\" 3             ADR 4 relates to ADR 3

CUSTOM REVERSE LINK:
  adrs link 3 Extends 1 \"Extended by\"    Specify custom reverse link

COMMON LINK TYPES (reverse auto-derived):
  Supersedes    ->  Superseded by
  Amends        ->  Amended by
  Relates to    ->  Relates to (symmetric)

The reverse link is automatically added to the target ADR.")]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs status 3 accepted                 Mark ADR 3 as accepted
  adrs status 2 deprecated               Mark ADR 2 as deprecated
  adrs status 1 superseded --by 5        Mark ADR 1 as superseded by ADR 5
  adrs status 4 rejected                 Mark ADR 4 as rejected
  adrs status 3 \"In Review\"              Use custom status

STANDARD STATUSES:
  proposed      Initial state (default for new ADRs)
  accepted      Decision has been approved
  deprecated    No longer recommended but not replaced
  superseded    Replaced by another ADR (use --by)
  rejected      Decision was not approved

Note: Use --by with 'superseded' to create a link to the replacing ADR.")]
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

    /// Generate shell completions
    #[command(after_long_help = "\
EXAMPLES:
  adrs completions bash > ~/.bash_completion.d/adrs
  adrs completions zsh > ~/.zfunc/_adrs
  adrs completions fish > ~/.config/fish/completions/adrs.fish
  adrs completions powershell > _adrs.ps1

BASH:
  Add to ~/.bashrc:
    source ~/.bash_completion.d/adrs

ZSH:
  Add to ~/.zshrc (before compinit):
    fpath=(~/.zfunc $fpath)
    autoload -Uz compinit && compinit

FISH:
  Completions are loaded automatically from ~/.config/fish/completions/")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: ShellArg,
    },

    /// Start MCP server for AI agent integration (requires --features mcp)
    #[cfg(feature = "mcp")]
    #[command(after_long_help = "\
Starts an MCP (Model Context Protocol) server on stdio for AI agent integration.

TOOLS PROVIDED:
  list_adrs     List all ADRs with optional status/tag filters
  get_adr       Get full content of an ADR by number
  search_adrs   Search ADRs for matching text

USAGE WITH CLAUDE:
  Add to your Claude Desktop config (claude_desktop_config.json):
  {
    \"mcpServers\": {
      \"adrs\": {
        \"command\": \"adrs\",
        \"args\": [\"mcp\", \"serve\"],
        \"cwd\": \"/path/to/your/project\"
      }
    }
  }

The server reads ADRs from the current working directory's repository.")]
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
}

#[cfg(feature = "mcp")]
#[derive(Subcommand)]
enum McpCommands {
    /// Start the MCP server on stdio
    Serve,
}

/// Shell types for completion generation
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ShellArg {
    /// Bourne Again Shell
    Bash,
    /// Z Shell
    Zsh,
    /// Fish Shell
    Fish,
    /// PowerShell
    Powershell,
    /// Elvish Shell
    Elvish,
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate a table of contents
    #[command(after_long_help = "\
EXAMPLES:
  adrs generate toc                      Generate markdown TOC
  adrs generate toc > doc/adr/README.md  Save to README
  adrs generate toc --ordered            Use numbered list (1. 2. 3.)
  adrs generate toc --prefix ./          Adjust link paths
  adrs generate toc --intro header.md    Prepend content from file")]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs generate graph                    Generate DOT format graph
  adrs generate graph | dot -Tpng > g.png  Render as PNG
  adrs generate graph --prefix https://example.com/adr/
                                         Add URLs to nodes
  adrs generate graph -e html            Use .html extension for links")]
    Graph {
        /// Prefix for node URLs
        #[arg(short, long, value_name = "PREFIX")]
        prefix: Option<String>,

        /// File extension for links [default: md]
        #[arg(short, long, default_value = "md")]
        extension: String,
    },

    /// Generate an mdbook
    #[command(after_long_help = "\
EXAMPLES:
  adrs generate book                     Generate in ./book directory
  adrs generate book -o docs/adr-book    Custom output directory
  adrs generate book -t \"Our ADRs\"       Set book title
  cd book && mdbook serve                Preview the generated book")]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs export json                       Export all ADRs as JSON array
  adrs export json --pretty              Pretty-printed JSON output
  adrs export json 3                     Export only ADR 3
  adrs export json --dir ./adrs          Export from directory (no repo needed)

FOR DOCUMENTATION/CATALOGS:
  adrs export json --metadata-only       Export metadata without full content
  adrs export json --base-url https://github.com/org/repo/blob/main/doc/adr
                                         Include source URLs in export

PIPING:
  adrs export json --pretty > adrs.json  Save to file
  adrs export json | jq '.[] | .title'   Process with jq")]
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
    #[command(after_long_help = "\
EXAMPLES:
  adrs import json adrs.json             Import from JSON file
  adrs import json --dry-run adrs.json   Preview without writing files
  adrs import json --overwrite adrs.json Replace existing ADRs
  cat adrs.json | adrs import json -     Import from stdin

MERGING REPOSITORIES:
  adrs import json --renumber external.json
                                         Append ADRs with new numbers
  adrs import json --renumber --dry-run external.json
                                         Preview renumbering

OPTIONS:
  --dry-run     See what would be imported without making changes
  --renumber    Assign new numbers starting after existing ADRs
                (also available as --append)
  --overwrite   Replace existing files instead of skipping
  --ng          Use YAML frontmatter in imported files")]
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
            no_edit,
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
                no_edit,
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
        Commands::Completions { shell } => {
            let shell = match shell {
                ShellArg::Bash => Shell::Bash,
                ShellArg::Zsh => Shell::Zsh,
                ShellArg::Fish => Shell::Fish,
                ShellArg::Powershell => Shell::PowerShell,
                ShellArg::Elvish => Shell::Elvish,
            };
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "adrs", &mut io::stdout());
            Ok(())
        }
        #[cfg(feature = "mcp")]
        Commands::Mcp { command } => match command {
            McpCommands::Serve => {
                let discovered = discover_or_error(&start_dir, cli.working_dir.is_some())?;
                tokio::runtime::Runtime::new()
                    .context("Failed to create tokio runtime")?
                    .block_on(mcp::serve(discovered.root))
                    .context("MCP server error")
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
