use std::fs::{read_dir, read_to_string};

use anyhow::Result;
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

    let superceded = args
        .superceded
        .iter()
        .map(|s| {
            let best_match = best_match(&adr_dir, s).unwrap();
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

    let title = args.title.join(" ");
    let number = next_adr_sequence(&adr_dir)?;
    let linked = args
        .link
        .iter()
        .map(|link| link.split(':').collect())
        .map(|parts: Vec<_>| {
            let linked_adr = best_match(&adr_dir, parts[0]).unwrap();
            let link_name = parts[1];
            let reverse_link = parts[2];

            let lines = read_to_string(linked_adr.clone())
                .unwrap()
                .lines()
                .map(String::from)
                .collect::<Vec<_>>();
            let first = lines.first().unwrap().clone();
            let parts = first.split_once(char::is_whitespace).unwrap();
            let filename = format!("{:0>4}-{}.md", number, adr_filename(&title));
            append_status(
                &linked_adr,
                format!("{} [{}. {}]({})", reverse_link, number, title, filename).as_str(),
            )
            .unwrap();

            (link_name.to_owned(), parts.1.to_string(), linked_adr)
        })
        .collect::<Vec<_>>();
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
