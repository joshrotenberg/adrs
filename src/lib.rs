//! ADR: Support library for parsing and rendering ADR files
use chrono::prelude::*;
use chrono::NaiveDate;
use heck::KebabCase;
use std::fs;
use std::path::PathBuf;

mod error;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

// By default, ADRs are created at the current level in the `doc/adr` directory.
const ADR_DEFAULT_DIRECTORY: &str = "doc/adr";
// const ADR_TEMPLATE_NYGARD: &'static str = include_str!("templates/adr-nygard.hbs");

// ADR statuses, including the option of a custom status.
#[derive(Debug, Clone)]
pub enum Status {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superceded,
    Custom(String),
}

// The default status for a new ADRS is `Proposed`.
impl Default for Status {
    fn default() -> Self {
        Status::Proposed
    }
}

// Struct representing an ADR for parsing and/or rendering.
#[derive(Debug, Clone)]
pub struct Adr<'a> {
    pub directory: &'a str,
    pub filename: Option<String>,
    pub index: u16,
    pub title: Option<&'a str>,
    pub date: NaiveDate,
    pub status: Status,
    pub context: Option<&'a str>,
    pub decision: Option<&'a str>,
    pub consequences: Option<&'a str>,
    is_parsed: bool,
    is_rendered: bool,
}

impl<'a> Default for Adr<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Adr<'a> {
    // Create a new ADR with the default status, default directory location and current date.
    pub fn new() -> Self {
        Self {
            directory: ADR_DEFAULT_DIRECTORY,
            filename: None,
            index: 0,
            title: None,
            date: Local::now().naive_local().date(),
            status: Status::default(),
            context: None,
            decision: None,
            consequences: None,
            is_parsed: false,
            is_rendered: false,
        }
    }

    // Set the top level directory for the ADR, overriding the default. This can be a relative or fully qualified path.
    pub fn directory(mut self, directory: &'a str) -> Self {
        self.directory = directory;
        self
    }

    // Set the ADR title. This field is required for rendering.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    // Set the ADR status. If not called, the ADR will have the default status when rendered.
    pub fn status(mut self, status: impl Into<Status>) -> Self {
        self.status = status.into();
        self
    }

    // Set the ADR's context text.
    pub fn context(mut self, context: &'a str) -> Self {
        self.context = Some(context);
        self
    }

    // Set the ADR's decision text.
    pub fn decision(mut self, decision: &'a str) -> Self {
        self.decision = Some(decision);
        self
    }

    // Set the ADR's consequences text.
    pub fn consequences(mut self, consequences: &'a str) -> Self {
        self.consequences = Some(consequences);
        self
    }

    pub fn parse() {}

    pub fn render(mut self) -> Result<()> {
        fs::create_dir_all(&self.directory)?;
        let current_adr_count = fs::read_dir(self.directory)?.count() as u16;
        self.index = current_adr_count + 1;
        self.filename = Some(format!(
            "{:04}-{}",
            self.index,
            self.title
                .expect("Title is required")
                .to_lowercase()
                .to_kebab_case()
        ));

        let mut full_path = PathBuf::new();

        full_path.push(self.directory);

        let mut full_path: PathBuf = [self.directory, self.filename.unwrap().as_ref()]
            .iter()
            .collect();
        full_path.set_extension("md");
        println!("{:?}", full_path);
        Ok(())
    }

    // Determines and sets the index based on the current count of ADR files, and generates and sets the filename.
    // fn generate_filename(mut self) -> Result<()> {
    //     // fs::create_dir_all(&self.directory)?;

    //     let count = fs::read_dir(&self.directory)?.count() as u16;
    //     self.index = count + 1;

    //     self.filename = Some(format!(
    //         "{:04}-{}.md",
    //         self.index.unwrap(),
    //         self.title
    //             .expect("Title is required")
    //             .to_lowercase()
    //             .to_kebab_case()
    //     ));
    //     println!("{:?}", self.filename);

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use crate::Adr;

    #[derive(Debug, Default)]
    pub struct What<'a> {
        doof: Option<&'a str>,
        cha: Option<&'a str>,
    }

    impl<'a> What<'a> {
        pub fn doof(mut self, doof: &'a str) -> Self {
            self.doof = Some(doof.as_ref());
            self
        }
    }

    // #[test]
    fn test_what() {
        let w = What::default().doof("what");
        println!("{:?}", w);

        let s = String::from("booo");
        let mut r = What::default().doof(s.as_ref());
        println!("{:?}", r);
    }

    // #[test]
    fn test_generate_filename() {
        let adr = Adr::new().directory("what").title("the").render();

        // .generate_filename();
        // let x = adr.filename;
        // adr.generate_filename();
        dbg!(adr);
    }
}
