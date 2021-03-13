use chrono::prelude::*;
use clap::ArgMatches;
use handlebars::Handlebars;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

const INIT_ADR_FILE_NAME: &'static str = "0001-record-architecture-decisions.md";

pub(crate) fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(directory) = matches.value_of("DIRECTORY") {
        fs::create_dir_all(directory)?;
        let path = Path::new(directory).join(INIT_ADR_FILE_NAME);

        if !path.exists() {
            let mut data: BTreeMap<String, String> = BTreeMap::new();
            data.insert(
                "date".to_string(),
                Local::now().format("%Y-%m-%d").to_string(),
            );

            let handlebars = Handlebars::new();
            let file = File::create(path)?;
            handlebars.render_template_to_write(
                crate::commands::NYGARD_TEMPLATE_INIT,
                &data,
                &file,
            )?;
        }
    }

    Ok(())
}
