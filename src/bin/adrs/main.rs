mod cli;
mod commands;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::build().get_matches();

    match matches.subcommand() {
        ("init", Some(matches)) => commands::init::run(matches)?,
        ("new", Some(matches)) => commands::new::run(matches)?,
        _ => panic!("oh no"),
    }

    Ok(())
}
