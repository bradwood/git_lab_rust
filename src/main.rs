//! [![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
//! [![coverage report](https://gitlab.com/bradwood/git-lab-rust/badges/master/coverage.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
//!
//! _ALPHA_ - what is here works, but functionality is still under active development.
//!
//! This is a cli tool that adds the `lab` command to `git` to enable interaction with a GitLab server.
//!
//! # Functionality
//!
//! The tool is designed to work as a custom command to the vanilla `git` cli command.
//!
//! ## Current functions
//!
//! * `init` -- initialise credentials aganst a remote GitLab server
//! * `project` -- interact with GitLab projects
//!     * `project create` -- create project
//!     * `project attach` -- associate a local repo with a project
//!     * `project (open|view|browse)` -- open project's URL in browser
//!
//! ## Planned functions
//!
//! * `issue` -- interact with issues
//! * `merge-request` -- interact with merge requests
//! * `pipeline` -- interact with Gitlab CI jobs
//! * probably others
//!
//! # Features
//!
//! ## Current features
//!
//! * Config stored using standard `git config` machinery
//! * JSON output in addition to plain text to allow for parsing with tools like `jq`.
//!
//! ## Planned features
//!
//! * `$EDITOR` integration
//! * Terminal-based markdown rendering
//!
//! # Installation
//!
//! For now, this is only available via `cargo` while under development. 
//!
//! ```
//! cargo install git_lab_cli
//! ```
//! # Compatibility
//!
//! Supports GitLab server version 13
//!
//! # Contributions
//!
//! Merge requests are welcome. Please raise a merge request on [GitLab](https://gitlab.com/bradwood/git-lab-rust), not GitHub.

#[macro_use]
extern crate log;
mod config;
mod subcommand;
mod utils;
mod gitlab;

mod cmds {
    pub mod init;
    pub mod merge_request;
    pub mod project;
}

use anyhow::Result;

use config::Config;

use crate::cmds::{init, merge_request, project};

fn main() -> Result<()> {
    let cli_commands = subcommand::ClapCommands {
        commands: vec![
            Box::new(init::Init {
                clap_cmd: clap::SubCommand::with_name("init"),
            }),
            Box::new(merge_request::MergeRequest {
                clap_cmd: clap::SubCommand::with_name("merge-request"),
            }),
            Box::new(project::Project {
                clap_cmd: clap::SubCommand::with_name("project"),
            }),
        ],
    };

    let matches = clap::App::new("git-lab")
        .setting(clap::AppSettings::VersionlessSubcommands)
        .setting(clap::AppSettings::ColoredHelp)
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A custom git command for interacting with a GitLab server")
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Set verbosity level")
                .multiple(true),
        )
        .subcommands(cli_commands.generate())
        .get_matches();

    loggerv::init_with_verbosity(matches.occurrences_of("verbose")).unwrap();

    trace!("Initialising config from disk");
    let config = Config::defaults();

    trace!("Dispatching to subcommand");

    trace!("Config = {:?}", config);

    match matches.subcommand() {
        ("init", Some(sub_args)) => cli_commands.commands[0].run(config, sub_args.clone())?,
        ("merge-request", Some(sub_args)) => cli_commands.commands[1].run(config, sub_args.clone())?,
        ("project", Some(sub_args)) => cli_commands.commands[2].run(config, sub_args.clone())?,
        _ => (), // clap should catch this before it ever fires
    }
    Ok(())
}
