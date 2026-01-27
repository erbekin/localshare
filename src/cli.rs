use clap::{Arg, Command, command};

pub fn get_command() -> Command {
    command!()
        .subcommand(
            Command::new("new")
                .about("Initialize a new server directory")
                .arg(
                    Arg::new("workdir")
                        .value_parser(clap::builder::PathBufValueParser::new())
                        .help("The path to the directory to initialize")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("run")
                .arg(
                    Arg::new("workdir")
                        .value_parser(clap::builder::PathBufValueParser::new())
                        .help("The path to the directory to be used by server")
                        .required(true),
                )
                .about("Run the app"),
        )
        .subcommand_required(true)
        .propagate_version(true)
}
