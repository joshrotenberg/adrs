use std::fs::{read_dir, read_to_string};

use anyhow::{bail, Result};
use clap::Args;
use edit::edit;
use pulldown_cmark::{Event, HeadingLevel, Tag};
use pulldown_cmark_to_cmark::cmark_resume;
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
    superceded: Vec<(String, String)>,
    linked: Vec<(String, String, String)>,
}

pub(crate) fn run(args: &NewArgs) -> Result<()> {
    let adr_dir = read_to_string(".adr-dir")?;

    let title = args.title.join(" ");
    let number = next_adr_sequence(&adr_dir)?;

    let superceded = create_superceded(&adr_dir, &args.superceded)?;
    tracing::debug!(?superceded);

    let linked = create_linked(&adr_dir, number, &title, &args.link)?;
    tracing::debug!(?linked);

    let new_context = NewAdrContext {
        number,
        date: now()?,
        title,
        superceded,
        linked,
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
    println!("{}", filename);

    Ok(())
}

fn create_superceded(adr_dir: &str, superceded: &Vec<String>) -> Result<Vec<(String, String)>> {
    let mut v = Vec::new();
    for s in superceded {
        let best_match = best_match(adr_dir, s)?;

        let title = get_adr_title(&best_match)?;
        v.push((title, best_match));
    }
    Ok(v)
}

fn create_linked(
    adr_dir: &str,
    number: i32,
    title: &str,
    linked: &Vec<String>,
) -> Result<Vec<(String, String, String)>> {
    let mut v = Vec::new();
    for s in linked {
        let parts = s.split(':').collect::<Vec<_>>();
        let linked_adr = best_match(adr_dir, parts[0])?;
        let linked_name = parts[1];
        let reverse_link = parts[2];
        tracing::debug!(?linked_adr);

        append_status(
            &linked_adr,
            format!("{} [{}. {}]({})", reverse_link, number, title, linked_adr).as_str(),
        )?;

        let title = get_adr_title(&linked_adr)?;
        v.push((linked_name.to_string(), title, linked_adr))
    }
    Ok(v)
}

fn get_adr_title(path: &str) -> Result<String> {
    let lines = read_to_string(path)?
        .lines()
        .map(String::from)
        .collect::<Vec<_>>();
    if let Some(first) = lines.first() {
        if let Some((_, rest)) = first.split_once(char::is_whitespace) {
            return Ok(rest.to_string());
        }
        bail!("Couldn't find a title for ADR");
    }
    bail!("Couldn't find a title for ADR");
}

fn best_match(path: &str, s: &str) -> Result<String> {
    let x = s.parse::<i32>();
    let r = match x {
        Ok(n) => best_match_i32(path, n),
        Err(_) => best_match_str(path, s),
    };
    tracing::debug!(?r);
    r
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

    let first = adrs.first().expect("No ADR matched");
    Ok(first.0.to_str().unwrap().to_string())
}

fn append_status(path: &str, status: &str) -> Result<()> {
    let markdown_input = read_to_string(path)?;
    let mut buf = String::with_capacity(markdown_input.len() + 128);

    let mut state = None;
    let mut in_status = false;
    for (event, offset) in pulldown_cmark::Parser::new(&markdown_input).into_offset_iter() {
        match event {
            Event::End(Tag::Heading(HeadingLevel::H2, _, _)) => {
                if markdown_input[offset].starts_with("## Status") {
                    in_status = true;
                }
            }
            Event::End(Tag::Paragraph) => {
                if in_status {
                    buf = buf + "\n\n" + status;
                }
                in_status = false;
            }
            _ => {}
        };
        state = cmark_resume(std::iter::once(event), &mut buf, state.take())?.into();
    }
    if let Some(state) = state {
        state.finalize(&mut buf)?;
    }
    std::fs::write(path, buf)?;
    Ok(())
}
