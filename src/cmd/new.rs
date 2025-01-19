use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use edit::edit;
use handlebars::Handlebars;
use serde::Serialize;

use crate::adr::{
    append_status, find_adr, find_adr_dir, format_adr_path, get_title, next_adr_number, now,
    remove_status,
};

static DEFAULT_NEW_TEMPLATE: &str = include_str!("../../templates/nygard/new.md");
static DEFAULT_CUSTOM_TEMPLATE_FILENAME: &str = "templates/template.md";

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
pub(crate) struct NewArgs {
    /// A reference to a previous decision to supersede with this new one
    #[arg(short, long, alias("superceded"))]
    superseded: Vec<String>,
    /// Link the new Architectural Decision to a previous Architectural Decision Record
    #[arg(short, long)]
    link: Vec<String>,
    /// Title of the new Architectural Decision Record
    #[arg(trailing_var_arg = true, required = true)]
    title: Vec<String>,
    /// Use a custom template when generating the new Architectural Decision Record.
    /// Relative paths are resolved with respect to the directory specified in `.adr-dir`.
    #[arg(env = "ADRS_TEMPLATE", short, long)]
    template: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct NewAdrContext {
    number: i32,
    title: String,
    date: String,
    superseded: Vec<String>,
    linked: Vec<String>,
}

pub(crate) fn run(args: &NewArgs) -> Result<()> {
    let adr_dir = find_adr_dir().context("No ADR directory found")?;
    let raw_template = if let Some(template) = &args.template {
        if !template.exists() {
            return Err(anyhow::anyhow!(
                "Template file not found: {}",
                template.display()
            ));
        }
        read_to_string(template)?
    } else if let Ok(template) = read_to_string(adr_dir.join(DEFAULT_CUSTOM_TEMPLATE_FILENAME)) {
        template
    } else {
        DEFAULT_NEW_TEMPLATE.to_string()
    };

    let title = args.title.join(" ");
    let number = next_adr_number(&adr_dir)?;

    let superseded = args
        .superseded
        .iter()
        .map(|adr| {
            let adr_path = find_adr(&adr_dir, adr).expect("No ADR found");
            let adr_title = get_title(&adr_path).expect("No title found");

            remove_status(&adr_path, "Accepted").expect("Unable to update status");
            format!(
                "Supersedes [{}]({})",
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
        superseded,
        linked,
    };

    let mut registry = Handlebars::new();
    registry.register_template_string("new_adr", raw_template)?;
    let rendered = registry.render("new_adr", &new_context)?;
    let edited = edit(rendered)?;

    std::fs::write(&path, edited)?;

    println!("{}", path.display());
    Ok(())
}
