mod create;
mod open;
mod quick_edit;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::config;
use crate::gitlab::MergeRequest as GLMergeRequest;
use crate::gitlab::MergeRequestBuilder;
use crate::gitlab;
use crate::subcommand;
use crate::utils;
use crate::utils::validator;
use crate::utils::ShortCmd;

#[derive(Debug, Deserialize)]
pub struct MergeRequest {
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: Option<String>,
    state: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    closed_by: Option<Map<String, Value>>,
    labels: Vec<String>,
    milestone: Option<String>,
    author: Map<String, Value>,
    assignees: Option<Vec<Map<String, Value>>>,
    user_notes_count: u64,
    upvotes: u64,
    downvotes: u64,
    discussion_locked: Option<bool>,
    web_url: String,
    task_completion_status: Option<Map<String, Value>>,
    references: Map<String, Value>,
    subscribed: Option<bool>,
}

pub fn generate_basic_mr_builder<'a>(
    args: &'a clap::ArgMatches,
    mr_arg_name: &str,
    config: &'a config::Config,
    m: &'a mut MergeRequestBuilder<'a>,
) -> Result<GLMergeRequest<'a>> {

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    m.project(project_id);
    m.merge_request(args.value_of(&mr_arg_name).unwrap().parse::<u64>().unwrap());
    m.build()
        .map_err(|e| anyhow!("Could not construct query for this merge request.\n {}",e))
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
                            .visible_alias("closes")
                            .short("i")
                            .help("Specifies which issue this merge request will close")
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
                        clap::Arg::with_name("desc")
                            .long("desc")
                            .short("d")
                            .help("Description")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("title")
                            .help("Merge request title")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("squash")
                            .help("Squash commits when merging")
                            .long("squash")
                            .short("q")
                            .takes_value(false)
                    )
                    .arg(
                        clap::Arg::with_name("remove_src")
                            .help("Remove source branch on successful merge")
                            .long("remove_src")
                            .short("r")
                            .takes_value(false)
                    )
                    .arg(
                        clap::Arg::with_name("checkout")
                            .help("Checkout branch of the created merge request")
                            .long("checkout")
                            .short("c")
                            .takes_value(false)
                    )
                    .arg(
                        clap::Arg::with_name("labels")
                            .long("labels")
                            .short("l")
                            .help("Sets merge request label(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    .arg(
                        clap::Arg::with_name("assignees")
                            .long("assignees")
                            .short("a")
                            .help("Username(s) of merge request assignee(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    .after_help(
"This command tries to do the right thing by taking into account local and remote repo, branch and \
commit state. It may create a source branch on the GitLab server if it cannot infer which to use. \
Where appropriate, it will prompt the user to input required information. It will also aim to \
follow GitLab conventions using branch names, merge request titles and trigger text within the \
merge request description. \
\
NB: The current implementation requires that the GitLab-hosted git remote is called `origin`."
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("unlock")
                    .about("Unlocks a merge request")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge Request ID to unlock")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for merge request in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("lock")
                    .about("Locks a merge request")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge request ID to lock")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for merge request in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("reopen")
                    .about("Re-opens a merge request")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge request ID to re-open")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for merge request in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("close")
                    .about("Closes a merge request")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge Request ID to close")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for merge request in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("open")
                    .about("Opens the merge request in the default browser")
                    .visible_aliases(&["view", "browse"])
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("url")
                            .short("u")
                            .long("print_url")
                            .help("Prints the URL instead of opening it.")
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for merge request in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge request ID to open")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .after_help(
"This command will open the default browser to the URL of the passed merge request. It will use the BROWSER \
environment variable to determine which browser to use. If this is not set, on Linux, it will \
try `xdg-open(1)`.",
                    ),
            )
    }

    fn run(&self, config: config::Config, args: clap::ArgMatches) -> Result<()> {
        trace!("Config: {:?}", config);
        trace!("Args: {:?}", args);

        let gitlabclient = gitlab::new(&config).context("Could not create GitLab client connection.")?;

        match args.subcommand() {
            ("create", Some(a)) => create::create_merge_request_cmd(a.clone(), config, *gitlabclient)?,
            ("open", Some(a)) => open::open_merge_request_cmd(a.clone(), config, *gitlabclient)?,
            // ("show", Some(a)) => show::show_issue_cmd(a.clone(), config, *gitlabclient)?,
            // ("list", Some(a)) => list::list_issues_cmd(a.clone(), config, *gitlabclient)?,
            // // ("status", Some(a)) => status::status_issues_cmd(a.clone(), config, *gitlabclient)?,
            ("close", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Close, config, *gitlabclient)?,
            ("reopen", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Reopen, config, *gitlabclient)?,
            ("lock", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Lock, config, *gitlabclient)?,
            ("unlock", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Unlock, config, *gitlabclient)?,
            _ => unreachable!(),
        }







        Ok(())
    }
}
