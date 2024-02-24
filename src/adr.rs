use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;
use time::macros::format_description;
use tinytemplate::TinyTemplate;

static TEMPLATE: &str = include_str!("../templates/nygard.md");

#[derive(Debug, Serialize)]
struct AdrContext {
    title: String,
    number: i32,
    date: String,
    status: String,
    context: String,
    decision: String,
    consequences: String,
}

pub struct AdrBuilder {
    title: Option<String>,
    status: Option<String>,
    context: Option<String>,
    decision: Option<String>,
    consequences: Option<String>,
}

impl AdrBuilder {
    pub fn new() -> AdrBuilder {
        AdrBuilder {
            title: None,
            status: None,
            context: None,
            decision: None,
            consequences: None,
        }
    }

    pub fn title(mut self, title: &str) -> AdrBuilder {
        self.title = Some(title.to_string());
        self
    }

    pub fn status(mut self, status: &str) -> AdrBuilder {
        self.status = Some(status.to_string());
        self
    }

    pub fn context(mut self, context: &str) -> AdrBuilder {
        self.context = Some(context.to_string());
        self
    }

    pub fn decision(mut self, decision: &str) -> AdrBuilder {
        self.decision = Some(decision.to_string());
        self
    }

    pub fn consequences(mut self, consequences: &str) -> AdrBuilder {
        self.consequences = Some(consequences.to_string());
        self
    }

    pub fn write(self, path: &PathBuf) -> Result<String> {
        let next_adr_number = next_adr_number(path)?;

        let filename = generate_filename(&self.title.clone().unwrap());

        let context = AdrContext {
            title: self.title.unwrap(),
            number: next_adr_number,
            date: get_adr_time()?,
            status: self.status.unwrap(),
            context: self.context.unwrap(),
            decision: self.decision.unwrap(),
            consequences: self.consequences.unwrap(),
        };
        let mut tt = TinyTemplate::new();
        tt.add_template("adr", TEMPLATE)?;
        let rendered = tt.render("adr", &context)?;
        let x = format!("{}/{:0>4}-{}.md", path.display(), next_adr_number, filename);
        std::fs::write(&x, rendered)?;
        Ok(x)
    }
}

fn get_adr_time() -> Result<String> {
    let now = time::OffsetDateTime::now_local()?;
    let x = now.format(format_description!("[year]-[month]-[day]"))?;
    Ok(x)
}

fn generate_filename(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
        .to_lowercase()
}

fn next_adr_number(path: impl AsRef<Path>) -> Result<i32> {
    let entries = std::fs::read_dir(path)?;
    let mut max = 0;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.starts_with(char::is_numeric) {
                if let Some((num, _rest)) = file_name.split_once('-') {
                    if let Ok(number) = num.parse::<i32>() {
                        if number > max {
                            max = number;
                        }
                    }
                }
            }
        }
    }
    Ok(max + 1)
}

#[cfg(test)]
mod tests {
    use assert_fs::TempDir;

    use super::*;

    #[test]
    fn test_generate_filename() {
        let title = "Record Architecture Decisions";
        let result = generate_filename(title);
        assert_eq!(result, "record-architecture-decisions");
    }

    #[test]
    fn test_next_adr_number() {
        let tmp_dir = TempDir::new().unwrap();
        let result = next_adr_number(tmp_dir.path());
        assert_eq!(result.unwrap(), 1);
    }
}
