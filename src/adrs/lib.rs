use std::path::PathBuf;

use chrono::NaiveDate;

#[derive(Debug)]
pub struct ADR {
    pub path: PathBuf,
    pub title: Option<String>,
    pub date: Option<NaiveDate>,
    pub status: Option<String>,
    pub context: Option<String>,
    pub decision: Option<String>,
    pub consequences: Option<String>
}

impl ADR {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: PathBuf::from(path.into()),
            title: None,
            date: None, 
            status: None,
            context: None,
            decision: None,
            consequences: None
        }
    }

    pub fn parse() {

    }

    pub fn render() {

    }
}
