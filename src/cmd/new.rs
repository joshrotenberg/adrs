use std::fs::read_to_string;

use anyhow::Result;
use clap::Args;
use edit::edit;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::adr::{append_status, find_adr, format_adr_path, get_title, next_adr_number, now};

static NEW_TEMPLATE: &str = include_str!("../../templates/nygard/new.md");

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub(crate) struct NewArgs {
    /// A reference to a previous decision to supercede with this new one
    #[arg(short, long)]
    superceded: Vec<String>,
    /// Link the new Architectural Decision to a previous Architectural Decision Record
    #[arg(short, long)]
    link: Vec<String>,
    /// Title of the new Architectural Decision Record
    #[arg(trailing_var_arg = true, required = true)]
    title: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NewAdrContext {
    number: i32,
    title: String,
    date: String,
    superceded: Vec<String>,
    linked: Vec<String>,
}

pub(crate) fn run(args: &NewArgs) -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;
    let title = args.title.join(" ");
    let number = next_adr_number(&adr_dir)?;

    let superceded = args
        .superceded
        .iter()
        .map(|adr| {
            let adr_path = find_adr(&adr_dir, adr).expect("No ADR found");
            let adr_title = get_title(&adr_path).expect("No title found");
            format!(
                "Supercedes [{}]({})",
                adr_title,
                adr_path.file_name().unwrap().to_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let path = format_adr_path(adr_dir.as_ref(), number, &title);
    let linked = args
        .link
        .iter()
        .map(|link| {
            let parts = link.split(':').collect::<Vec<_>>();
            let source_filename = &path.file_name().unwrap().to_str().unwrap();
            let source_title = format!("{}. {}", number, &title);

            let target_link = format!("{} [{}]({})", parts[2], source_title, source_filename);
            let target_filename = find_adr(&adr_dir, parts[0]).expect("No ADR found");
            let target_title = get_title(&target_filename).expect("No ADR found");

            append_status(&target_filename, &target_link).expect("Unable to append status");

            let source_link = format!(
                "{} [{}]({})",
                parts[1],
                target_title,
                target_filename.file_name().unwrap().to_str().unwrap(),
            );

            source_link
        })
        .collect::<Vec<_>>();

    let new_context = NewAdrContext {
        number,
        date: now()?,
        title: title.clone(),
        superceded,
        linked,
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("new_adr", NEW_TEMPLATE)?;
    let rendered = tt.render("new_adr", &new_context)?;
    let edited = edit(rendered)?;

    std::fs::write(&path, edited)?;

    println!("{}", path.display());
    Ok(())
}
