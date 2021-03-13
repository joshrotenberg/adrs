use clap::ArgMatches;
use std::error::Error;

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    println!("in new");

    Ok(())
}
