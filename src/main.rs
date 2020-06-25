//! [![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
//! [![Crates.io](https://img.shields.io/crates/v/git_lab_cli)](https://crates.io/crates/git_lab_cli)
//! [![Crates.io](https://img.shields.io/crates/d/git_lab_cli)](https://crates.io/crates/git_lab_cli)
//! [![musl binaries](https://img.shields.io/badge/musl%20binary-download-brightgreen)](https://gitlab.com/bradwood/git-lab-rust/-/releases)
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
//!  * `init` -- initialise credentials against a remote GitLab server
//!  * `project` -- interact with GitLab projects
//!     * `project create` -- create project
//!     * `project attach` -- associate a local repo with a project
//!     * `project (open|view|browse)` -- open project's URL in browser
//!     * `project (show|info|get)` -- show details about a project
//!  * `issue` -- interact with issues
//!     * `issue create` -- create issue (either entirely via cli-passed parameters, or
//!        interactively, by prompting the user for the inputs needed)
//!     * `issue (open|view|browse)` -- open issue's URL in browser
//!     * `issue (show|info|get)` -- show details about a issue
//!     * `issue list` -- get list of issues
//!
//! ## Planned functions
//!
//!  * `project list` -- get list of projects
//!  * `merge-request` -- interact with merge requests
//!  * `pipeline` -- interact with Gitlab CI jobs
//!  * `group` -- interact with Gitlab groups
//!  * `user` -- interact with Gitlab users
//!  * probably others like `labels`, etc..
//!
//! # Features
//!
//! ## Current features
//!
//!  * Config stored using standard `git config` machinery
//!  * Locally cached Gitlab metadata to improve usability when creating gitlab objects
//!    interactively
//!  * JSON output in addition to plain text to allow for parsing with tools like `jq`
//!  * Terminal-based markdown rendering
//!  * `$EDITOR` integration on `create` commands
//!  * `musl` binaries available [here](https://gitlab.com/bradwood/git-lab-rust/-/releases)
//!
//! ## Planned features
//!
//!  * support for various linux packaging tools like AUR, Deb, RPM, etc
//!  * non-Linux support (maybe)
//!
//! # Installation
//!
//! ## Cargo
//!
//! To install via `cargo`:
//!
//! ```
//! cargo install git_lab_cli
//! ```
//! ## Statically linked Linux binaries
//!
//! Grab a tarball for these [here](https://gitlab.com/bradwood/git-lab-rust/-/releases).
//!
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
    pub mod issue;
    pub mod merge_request;
    pub mod project;
}

use anyhow::{anyhow, Result};

use config::Config;

use crate::cmds::{init, merge_request, project, issue};

/// This should be called before calling any cli method or printing any output.
/// See https://github.com/rust-lang/rust/issues/46016#issuecomment-605624865
fn reset_signal_pipe_handler() -> Result<()> {
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;

        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)
                .map_err(|e| anyhow!(e.to_string()))?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {

    reset_signal_pipe_handler()?;

    //TODO: refactor this at some point...
    let cli_commands = subcommand::ClapCommands {
        commands: vec![
            Box::new(init::InitCmd {
                clap_cmd: clap::SubCommand::with_name("init"),
            }),
            Box::new(merge_request::MergeRequestCmd {
                clap_cmd: clap::SubCommand::with_name("merge-request"),
            }),
            Box::new(issue::IssueCmd {
                clap_cmd: clap::SubCommand::with_name("issue"),
            }),
            Box::new(project::ProjectCmd {
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
        .after_help("Please report bugs at https://gitlab.com/bradwood/git-lab-rust")
        .get_matches();

    loggerv::init_with_verbosity(matches.occurrences_of("verbose")).unwrap();

    trace!("Initialising config from disk");
    let config = Config::defaults();

    trace!("Dispatching to subcommand");

    trace!("Config = {:?}", config);

    match matches.subcommand() {
        ("init", Some(sub_args)) => cli_commands.commands[0].run(config, sub_args.clone())?,
        ("merge-request", Some(sub_args)) => cli_commands.commands[1].run(config, sub_args.clone())?,
        ("issue", Some(sub_args)) => cli_commands.commands[2].run(config, sub_args.clone())?,
        ("project", Some(sub_args)) => cli_commands.commands[3].run(config, sub_args.clone())?,
        _ => (), // clap should catch this before it ever fires
    }
    Ok(())
}
