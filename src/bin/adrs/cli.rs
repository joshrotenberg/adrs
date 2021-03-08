use clap::{App, AppSettings, Arg, SubCommand};

pub fn build() -> App<'static, 'static> {
    App::new("adrs")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize a new ADR directory")
                .arg(
                    Arg::with_name("DIRECTORY")
                        .default_value("doc/adr")
                        .help("specify the ADR directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new, numbered ADR")
                .arg(Arg::with_name("TITLE").required(true))
                .arg(
                    Arg::with_name("SUPERCEDED")
                        .short("s")
                        .long("superceded")
                        .takes_value(true)
                        .multiple(true)
                        .help("Reference to superceded ADR"),
                )
                .arg(
                    Arg::with_name("LINK")
                        .short("l")
                        .long("link")
                        .takes_value(true)
                        .multiple(true)
                        .help("Link to a previous ADR"),
                ),
        )
}
