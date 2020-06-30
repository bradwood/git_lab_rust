mod create;

use anyhow::{Context, Result};
// use chrono::{DateTime, Utc, NaiveDate};
use serde::Deserialize;
// use serde_json::{Map, Value};

use crate::config;
// use crate::gitlab::Issue as GLIssue;
// use crate::gitlab::IssueBuilder;
use crate::gitlab;
use crate::subcommand;
use crate::utils::validator;
// use crate::utils;

#[derive(Debug, Deserialize)]
pub struct MergeRequest {

}

pub struct MergeRequestCmd<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl subcommand::SubCommand for MergeRequestCmd<'_> {
    fn gen_clap_command(&self) -> clap::App {
        let c = self.clap_cmd.clone();
        c.about("Creates, manipulates and queries merge requests")
            .setting(clap::AppSettings::ColoredHelp)
            .setting(clap::AppSettings::VersionlessSubcommands)
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("create")
                    .about("Creates a merge request")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("project_id")
                            .long("project_id")
                            .short("p")
                            .help("Project id - defaults to the attached project")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("issue_id")
                            .long("issue_id")
                            .short("i")
                            .help("Creates merge request related to this issue")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("source_branch")
                            .long("source")
                            .short("s")
                            .help("Source branch")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("target_branch")
                            .long("target")
                            .short("t")
                            .help("Target branch")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("title")
                            .help("Merge request title")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .after_help(
"Help..."
                    ),
            )
    }

    fn run(&self, config: config::Config, args: clap::ArgMatches) -> Result<()> {
        trace!("Config: {:?}", config);
        trace!("Args: {:?}", args);

        let gitlabclient = gitlab::new(&config).context("Could not create GitLab client connection.")?;

        match args.subcommand() {
            ("create", Some(a)) => create::create_merge_request_cmd(a.clone(), config, *gitlabclient)?,
            // ("open", Some(a)) => open::open_issue_cmd(a.clone(), config, *gitlabclient)?,
            // ("show", Some(a)) => show::show_issue_cmd(a.clone(), config, *gitlabclient)?,
            // ("list", Some(a)) => list::list_issues_cmd(a.clone(), config, *gitlabclient)?,
            // // ("status", Some(a)) => status::status_issues_cmd(a.clone(), config, *gitlabclient)?,
            // ("close", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Close, config, *gitlabclient)?,
            // ("reopen", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Reopen, config, *gitlabclient)?,
            // ("lock", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Lock, config, *gitlabclient)?,
            // ("unlock", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Unlock, config, *gitlabclient)?,
            _ => unreachable!(),
        }







        Ok(())
    }
}
