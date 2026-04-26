use clap::{Arg, Command, command};

pub fn get_command() -> Command {
    command!()
        .subcommand(
            Command::new("new")
                .about("Set up a new LocalShare server directory")
                .long_about(
                    "Creates a new directory at the given path and initialises it for use \
                     with LocalShare.\n\n\
                     The following structure will be generated:\n  \
                     <workdir>/\n  \
                     ├── LocalShare.toml   (default server configuration)\n  \
                     ├── static/           (embedded UI assets)\n  \
                     └── uploads/          (directory for received files)\n\n\
                     Once initialised, start the server with:\n  \
                     localshare run <workdir>",
                )
                .arg(
                    Arg::new("workdir")
                        .value_parser(clap::builder::PathBufValueParser::new())
                        .help("Path of the directory to create and initialise")
                        .long_help(
                            "Path where the new server directory will be created. \
                             The directory must not already exist.",
                        )
                        .required(true),
                )
                .arg(
                    Arg::new("auth")
                        .long("auth")
                        .action(clap::ArgAction::SetTrue)
                        .help("Enable password authentication for admin actions")
                        .long_help(
                            "When set, the server will require a password to perform \
                             privileged actions such as deleting files.\n\n\
                             The password is read from the LOCALSHARE_PASSWORD environment \
                             variable each time the server starts. If the variable is not \
                             set or is empty, the server will refuse to start.",
                        ),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Start the LocalShare server")
                .long_about(
                    "Launches the LocalShare file-sharing server using the configuration \
                     found inside the specified directory.\n\n\
                     On startup the server will:\n  \
                     • Read LocalShare.toml for port, upload limits, and other settings\n  \
                     • Register itself on the local network via mDNS so nearby devices \
                       can discover it automatically\n  \
                     • Print a QR code to the terminal for quick mobile access\n\n\
                     The directory must have been initialised first with:\n  \
                     localshare new <workdir>",
                )
                .arg(
                    Arg::new("workdir")
                        .value_parser(clap::builder::PathBufValueParser::new())
                        .help("Path to an initialised LocalShare server directory")
                        .long_help(
                            "Path to a directory that was previously set up with \
                             'localshare new'. The directory must contain a valid \
                             LocalShare.toml configuration file.",
                        )
                        .required(true),
                ),
        )
        .subcommand_required(true)
        .propagate_version(true)
}