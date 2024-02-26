use std::fs::{read_dir, read_to_string};

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
    superceded: Vec<String>,
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
    superceded: Vec<(String, String)>,
}

pub(crate) fn run(args: &NewArgs) -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;

    let superceded = args
        .superceded
        .iter()
        .map(|s| {
            let best_match = best_match(&adr_dir, &s).unwrap();
            let lines = read_to_string(best_match.clone())
                .unwrap()
                .lines()
                .map(String::from)
                .collect::<Vec<_>>();
            let first = lines.first().unwrap().clone();

            let parts = first.split_once(char::is_whitespace).unwrap();
            (parts.1.to_string(), best_match)
        })
        .collect::<Vec<(_, _)>>();
    tracing::debug!(?superceded);

    let new_context = NewAdrContext {
        number: next_adr_sequence(&adr_dir)?,
        date: now()?,
        title: args.title.join(" "),
        superceded: superceded,
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

fn best_match(path: &str, s: &str) -> Result<String> {
    let x = s.parse::<i32>();
    match x {
        Ok(n) => best_match_i32(path, n),
        Err(_) => best_match_str(path, s),
    }
}

fn best_match_i32(path: &str, n: i32) -> Result<String> {
    let target = format!("{}/{:0>4}-", path, n);
    let adrs = read_dir(path)?
        .map(|entry| entry.unwrap().path())
        .filter(|filename| filename.is_file())
        .collect::<Vec<_>>();
    let m = adrs
        .iter()
        .find(|filename| filename.to_str().unwrap().starts_with(&target));
    Ok(m.unwrap().to_str().unwrap().to_string())
}

fn best_match_str(path: &str, s: &str) -> Result<String> {
    let mut adrs = read_dir(path)?
        .map(|entry| entry.unwrap().path())
        .map(|filename| {
            (
                filename.clone(),
                strsim::normalized_damerau_levenshtein(s, filename.to_str().unwrap()),
            )
        })
        .collect::<Vec<(_, _)>>();
    adrs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    tracing::debug!(?adrs);
    let first = adrs.first().unwrap();
    Ok(first.0.to_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {}
