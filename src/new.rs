use std::fs::read_to_string;

use anyhow::Result;
use clap::Args;
use edit::edit;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{adr_filename, next_adr_sequence, now};

static NEW_TEMPLATE: &str = include_str!("../templates/nygard/new.md");

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub(crate) struct NewArgs {
    /// A reference to a previous decision to supercede with this new one
    #[arg(short, long)]
    superceded: Option<Vec<String>>,
    /// Link the new Architectural Decision to a previous Architectural Decision Record
    #[arg(short, long)]
    link: Option<Vec<String>>,
    /// Title of the new Architectural Decision Record
    #[arg(trailing_var_arg = true, required = true)]
    title: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NewAdrContext {
    number: i32,
    title: String,
    date: String,
}

pub(crate) fn run(args: &NewArgs) -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;

    let new_context = NewAdrContext {
        number: next_adr_sequence(&adr_dir)?,
        date: now()?,
        title: args.title.join(" "),
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("new_adr", NEW_TEMPLATE)?;
    let rendered = tt.render("new_adr", &new_context)?;
    let edited = edit(rendered)?;

    let filename = format!(
        "{}/{:0>4}-{}.md",
        adr_dir,
        new_context.number,
        adr_filename(&new_context.title),
    );
    std::fs::write(&filename, edited)?;

    tracing::debug!("Created {}", filename);

    Ok(())
}
