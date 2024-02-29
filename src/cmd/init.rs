use std::{fs::create_dir_all, path::PathBuf};

use anyhow::Result;
use clap::Args;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{adr_filename, next_adr_sequence, now};

static INIT_TEMPLATE: &str = include_str!("../../templates/nygard/init.md");

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub(crate) struct InitArgs {
    /// Directory to initialize
    #[arg(default_value = "doc/adr")]
    pub(crate) directory: PathBuf,
}

#[derive(Debug, Serialize)]
struct InitAdrContext {
    number: i32,
    date: String,
}

pub(crate) fn run(args: &InitArgs) -> Result<()> {
    create_dir_all(&args.directory)?;

    let filename = format!(
        "{}/{:0>4}-{}.md",
        args.directory.to_str().unwrap(),
        next_adr_sequence(&args.directory)?,
        adr_filename("Record architecture decisions")
    );

    let init_context = InitAdrContext {
        number: next_adr_sequence(&args.directory)?,
        date: now()?,
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("init_adr", INIT_TEMPLATE)?;
    let rendered = tt.render("init_adr", &init_context)?;
    std::fs::write(&filename, rendered)?;

    tracing::debug!("Created {}", filename);

    std::fs::write(
        std::env::current_dir()?.join(".adr-dir"),
        args.directory.to_str().unwrap(),
    )?;

    tracing::debug!("Wrote .adr-dir");

    Ok(())
}
