use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::adr::{find_adr_dir, get_title, list_adrs};

static BOOK_TOML_TEMPLATE: &str = include_str!("../../../templates/book/book.toml");
static BOOK_SUMMARY_TEMPLATE: &str = include_str!("../../../templates/book/SUMMARY.md");

#[derive(Debug, Args)]
pub(crate) struct BookArgs {
    /// Target path for the book directory
    #[clap(long, short, default_value = "book")]
    path: PathBuf,
    /// Overwrite existing directory
    #[clap(long, short, default_value_t = false)]
    overwrite: bool,
    /// Title of the book
    #[clap(long, short, default_value = "Architecture Decision Records")]
    title: String,
    /// Description of the book
    #[clap(
        long,
        short,
        default_value = "A collection of architecture decision records"
    )]
    description: String,
    /// Author of the book
    #[clap(long, short)]
    author: Option<String>,
}

#[derive(Debug, Serialize)]
struct BookTomlContext {
    title: String,
    description: String,
    author: String,
}

#[derive(Debug, Serialize)]
struct SummaryContext {
    adrs: Vec<String>,
}

pub fn run_book(args: &BookArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;
    if args.path.exists() && !args.overwrite {
        anyhow::bail!(
            "Directory already exists: {}. Use the --overwrite flat to overwrite it.",
            args.path.display()
        );
    }

    create_dir_all(args.path.as_path().join("src"))?;
    let author = if let Some(author) = &args.author {
        author.clone()
    } else {
        format!(
            "{} <{}@{}>",
            whoami::realname(),
            whoami::username(),
            whoami::fallible::hostname().unwrap()
        )
    };

    let mut tt = TinyTemplate::new();

    let book_toml_context = BookTomlContext {
        title: args.title.clone(),
        description: args.description.clone(),
        author,
    };

    let book_toml = tt
        .add_template("book_toml", BOOK_TOML_TEMPLATE)
        .and_then(|_| tt.render("book_toml", &book_toml_context))
        .context("Unable to render book.toml template")?;

    std::fs::write(args.path.as_path().join("book.toml"), book_toml)?;

    let mut adr_titles = Vec::new();
    let adrs = list_adrs(Path::new(&adr_dir))?;
    for adr in adrs {
        std::fs::copy(
            &adr,
            args.path
                .as_path()
                .join("src")
                .join(adr.file_name().unwrap()),
        )?;
        let adr_title = get_title(adr.as_path())?;
        let (_number, title) = adr_title.split_once(char::is_whitespace).unwrap();
        let item = format!(
            "[{}]({})",
            title,
            adr.file_name().unwrap().to_str().unwrap()
        );
        adr_titles.push(item);
    }

    let summary_context = SummaryContext { adrs: adr_titles };

    let summary_mardkown = tt
        .add_template("SUMMARY.md", BOOK_SUMMARY_TEMPLATE)
        .and_then(|_| tt.render("SUMMARY.md", &summary_context))
        .context("Unable to render SUMMARY.md template")?;

    std::fs::write(
        args.path.as_path().join("src").join("SUMMARY.md"),
        summary_mardkown,
    )?;

    Ok(())
}
