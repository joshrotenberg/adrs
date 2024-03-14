use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::adr::{format_adr_path, next_adr_number, now};

static INIT_TEMPLATE: &str = include_str!("../../templates/nygard/init.md");

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub(crate) struct InitArgs {
    /// Directory to initialize
    #[arg(default_value = "doc/adr")]
    directory: PathBuf,
}

#[derive(Debug, Serialize)]
struct InitAdrContext {
    number: i32,
    date: String,
}

pub(crate) fn run(args: &InitArgs) -> Result<()> {
    create_dir_all(&args.directory)
        .with_context(|| format!("Unable to create {}", args.directory.display()))?;

    let number = next_adr_number(Path::new(&args.directory))
        .context("Unable to determine next ADR number")?;

    let title = "Record architecture decisions";

    let filename = format_adr_path(&args.directory, number, title);

    let init_context = InitAdrContext {
        number,
        date: now()?,
    };

    std::fs::write(
        std::env::current_dir()?.join(".adr-dir"),
        args.directory.to_str().unwrap(),
    )?;

    let mut tt = TinyTemplate::new();
    tt.add_template("init_adr", INIT_TEMPLATE)?;
    let rendered = tt
        .render("init_adr", &init_context)
        .context("Unable to render template")?;
    std::fs::write(&filename, rendered)
        .with_context(|| format!("Unable to write ADR file: {}", filename.display()))?;

    println!("{}", filename.display());

    Ok(())
}
