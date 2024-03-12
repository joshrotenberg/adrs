use std::fs::{create_dir_all, read_dir, read_to_string};
use std::path::{Path, PathBuf};

use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};
use pulldown_cmark_to_cmark::cmark_resume;
use time::macros::format_description;

// format the current date
pub(crate) fn now() -> Result<String> {
    let now = time::OffsetDateTime::now_local()?;
    let x = now.format(format_description!("[year]-[month]-[day]"))?;
    Ok(x)
}

// format the ADR path
pub(crate) fn format_adr_path(adr_dir: &Path, sequence: i32, title: &str) -> PathBuf {
    Path::new(adr_dir).join(format!(
        "{:0>4}-{}.md",
        sequence,
        title
            .split(|c| char::is_ascii_whitespace(&c) || char::is_ascii_punctuation(&c))
            .collect::<Vec<&str>>()
            .join("-")
            .to_lowercase()
    ))
}

// find the adr file that best matches the given string
pub(crate) fn find_adr<P: AsRef<Path>>(path: P, s: &str) -> Result<PathBuf> {
    if s.chars().all(char::is_numeric) {
        let n = s.parse::<i32>().expect("Invalid ADR number");
        find_adr_by_number(path.as_ref(), n)
    } else {
        find_adr_by_str(path.as_ref(), s)
    }
}

// takes the top level directory and a string to match and returns the best matching filename
pub(crate) fn find_adr_by_str(path: &Path, s: &str) -> Result<PathBuf> {
    let matcher = SkimMatcherV2::default();

    let mut adrs = list_adrs(path)?
        .into_iter()
        .filter_map(|filename| {
            matcher
                .fuzzy_match(filename.to_str().unwrap(), s)
                .map(|score| (filename, score))
        })
        .collect::<Vec<(_, _)>>();

    adrs.sort_by(|a, b| match b.1.partial_cmp(&a.1) {
        Some(x) => x,
        None => std::cmp::Ordering::Equal,
    });

    let first = adrs.first().expect("No ADR matched");
    Ok(first.0.clone())
}

// takes the top level directory and a number to match and returns the best matching filename
pub(crate) fn find_adr_by_number(path: &Path, n: i32) -> Result<PathBuf> {
    let target = path.join(format!("{:0>4}-", n));

    let target = target.to_str().expect("ADR path is not valid");

    let adrs = list_adrs(path)?;
    let m = adrs
        .iter()
        .find(|filename| filename.to_str().unwrap().starts_with(target));
    match m {
        None => {
            let msg = format!("No ADR found for {}", n);
            Err(anyhow::anyhow!(msg))
        }
        Some(x) => Ok(x.clone()),
    }
}

// returns a sorted list of all the ADRs in the directory
pub(crate) fn list_adrs(path: &Path) -> Result<Vec<PathBuf>> {
    let mut adrs = read_dir(path)?
        .map(|entry| entry.unwrap().path())
        .filter(|filename| {
            filename
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(char::is_numeric)
                && filename.is_file()
        })
        .collect::<Vec<_>>();

    adrs.sort();
    Ok(adrs)
}

// returns the title of the ADR
pub(crate) fn get_title(path: &Path) -> Result<String> {
    let markdown = std::fs::read_to_string(path)?;
    let parser = Parser::new(&markdown);
    let mut in_title = false;
    for event in parser {
        match event {
            Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) => {
                in_title = true;
            }
            Event::Text(text) => {
                if in_title {
                    return Ok(text.to_string());
                }
            }
            _ => {}
        }
    }
    Err(anyhow::anyhow!("No title found for ADR"))
}

// get the statuses of the ADR
pub(crate) fn get_status(path: &Path) -> Result<Vec<String>> {
    let markdown = std::fs::read_to_string(path)?;
    let parser = Parser::new(&markdown).into_offset_iter();
    let mut in_status = false;
    let mut buf = String::new();
    for (event, offset) in parser {
        match event {
            Event::Start(Tag::Heading(HeadingLevel::H2, _, _)) => {
                in_status = markdown[offset].starts_with("## Status");
            }
            Event::Start(Tag::Paragraph) => {
                if in_status {
                    buf += &markdown[offset];
                }
            }
            _ => {}
        }
    }
    Ok(buf.lines().map(|s| s.to_string()).collect())
}

// get only the statuses that are links
pub(crate) fn get_links(path: &Path) -> Result<Vec<(String, String, String)>> {
    let status = get_status(path)?;
    let mut links = Vec::new();
    for s in &status {
        let link = Parser::new(s).collect::<Vec<_>>();
        if link.len() > 3 {
            for event in &link {
                if let Event::Start(Tag::Link(_, _, _)) = event {
                    let parts = s
                        .split(&['[', ']', '(', ')'])
                        .filter(|&s| !s.is_empty())
                        .map(|s| s.trim())
                        .collect::<Vec<_>>();
                    let d = (
                        parts[0].to_owned(),
                        parts[1].to_owned(),
                        parts[2].to_owned(),
                    );
                    links.push(d)
                }
            }
        }
    }
    Ok(links)
}

// append the status to the ADR
pub(crate) fn append_status(path: &Path, status: &str) -> Result<()> {
    let markdown_input = std::fs::read_to_string(path)?;
    let mut buf = String::with_capacity(markdown_input.len() + status.len() + 2);

    let mut state = None;
    let mut in_status = false;
    for (event, offset) in Parser::new(&markdown_input).into_offset_iter() {
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

// read the .adr-dir file
pub(crate) fn read_adr_dir_file() -> Result<PathBuf> {
    let dir = read_to_string(".adr-dir")?;
    Ok(PathBuf::from(dir.trim()))
}

// find the ADR directory, defaulting to "doc/adr" and creating it if it doesn't exist
pub(crate) fn find_adr_dir() -> Result<PathBuf> {
    match read_adr_dir_file() {
        Ok(dir) => Ok(dir),
        _ => {
            create_dir_all("doc/adr")?;
            Ok(PathBuf::from("doc/adr"))
        }
    }
}
// get the next ADR number
pub(crate) fn next_adr_number(path: impl AsRef<Path>) -> Result<i32> {
    let adrs = list_adrs(path.as_ref())?;
    Ok(adrs.len() as i32 + 1)
}

#[cfg(test)]
mod tests {}
