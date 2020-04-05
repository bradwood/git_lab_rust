//! This is a cli tool that adds the `lab` command to `git` to enable interaction with a GitLab server.
//!
//! # Features
//!
//! The tool is designed to work as a custom command to the vanilla `git` cli command. Current
//! feature include:
//! * `init` -- initialite credentials aganst a remote GitLab server
//! * `merge-request` -- create and manipulate merge requests
//!
//! `git-lab` by default stores it's config using standard `git config` machinery.
//!
//! # Installation
//!
//! TBC
//!
//! # Contributions
//!
//! Merge requests are welcome.
//!
//! TOOO: Add more on build, test, and release machinery later.
mod config;
mod subcommand;
mod utils;

mod cmds {
    pub mod init;
    pub mod merge_request;
}

use config::Config;

use crate::cmds::{init, merge_request};

fn main() {
    let cli_commands = subcommand::ClapCommands {
        commands: vec![
            Box::new(init::Init {
                clap_cmd: clap::SubCommand::with_name("init"),
            }),
            Box::new(merge_request::MergeRequest {
                clap_cmd: clap::SubCommand::with_name("merge-request"),
            }),
        ],
    };

    let matches = clap::App::new("git-lab")
        .setting(clap::AppSettings::ColoredHelp)
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A git custom command for interacting with a GitLab server")
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Set verbosity level")
                .multiple(true),
        )
        .subcommands(cli_commands.generate())
        .get_matches();

    // Get vebosity
    let verbosity: usize = match matches.occurrences_of("v") {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3, _ => 3, };

    println!("Matches = {:#?}", matches);

    let config = Config::defaults();

    println!("Config = {:#?}", config);

    // Dispatch handler for passed command
    // TODO: Make this idomatic don't refer to specific entries in the vector which is ugly
    match matches.subcommand() {
        ("init", Some(sub_m)) => cli_commands.commands[0].run(),
        ("merge-request", Some(sub_m)) => cli_commands.commands[1].run(),
        _ => println!("{}", matches.usage()),
    }
}
