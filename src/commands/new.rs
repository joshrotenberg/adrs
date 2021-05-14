use clap::ArgMatches;
use heck::KebabCase;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub(crate) fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let title = matches.value_of("TITLE").unwrap();
    let directory = matches.value_of("DIRECTORY").unwrap();
    let count = fs::read_dir(directory)?.count();

    let filename = format!(
        "{:04}-{title}",
        count + 1,
        title = title.to_ascii_lowercase().to_kebab_case()
    );

    let mut path: PathBuf = [directory, filename.as_ref()].iter().collect();
    path.set_extension("md");

    dbg!(path);

    if let Some(title) = matches.value_of("TITLE") {
        // create a new adr from the template with the given title, incrementing the index appropriately.
        // if any superceded ADRs are specified, update the status of those ADRs to specify that they've been superceded by the new ADR.
    }

    Ok(())
}
