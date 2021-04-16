use clap::ArgMatches;
use std::error::Error;

pub(crate) fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(title) = matches.value_of("TITLE") {
        // create a new adr from the template with the given title, incrementing the index appropriately.
        // if any superceded ADRs are specified, update the status of those ADRs to specify that they've been superceded by the new ADR.
        dbg!(title);
    }

    Ok(())
}
