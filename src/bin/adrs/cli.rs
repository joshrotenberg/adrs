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
                        .default_value(crate::commands::ADR_DEFAULT_DIRECTORY)
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
        .subcommand(
            SubCommand::with_name("link")
                .about("Link together two ADRs")
                .arg(
                    Arg::with_name("SOURCE")
                        .short("s")
                        .long("source")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("TARGET")
                        .short("t")
                        .long("target")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("LINK")
                        .short("l")
                        .long("link")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("REVERSE-LINK")
                        .short("r")
                        .long("reverse-link")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("List the ADRs"))
        .subcommand(SubCommand::with_name("config").about("Show configuration"))
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate summary documentation")
                .arg(
                    Arg::with_name("TYPE")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["toc", "graph"]),
                ),
        )
}
