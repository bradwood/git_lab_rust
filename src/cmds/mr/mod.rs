mod create;
mod checkout;
mod open;
mod list;
mod quick_edit;
mod show;

use std::process::{Command, Stdio};

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
    merged_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    closed_by: Option<Map<String, Value>>,
    merged_by: Option<Map<String, Value>>,
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
    target_branch: String,
    source_branch: String,
    work_in_progress: bool,
    merge_when_pipeline_succeeds: bool,
    merge_status: String,
    has_conflicts: bool,
    blocking_discussions_resolved: bool,
    squash: bool,
}
pub fn checkout_mr(source_branch: &str) -> Result<()> {

    Command::new("git")
        .args(&["fetch","origin"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?;

    Command::new("git")
        .args(&["checkout","-b", source_branch, &("origin/".to_string() + source_branch)])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?;
    Ok(())
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
                clap::SubCommand::with_name("list")
                    .about("Lists merge requests")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("state")
                            .long("state")
                            .short("s")
                            .help("Filter merge requests by state")
                            .takes_value(true)
                            .possible_values(&["all", "closed", "opened", "merged", "locked"])
                            .default_value("opened")
                    )
                    .arg(
                        clap::Arg::with_name("scope")
                            .long("scope")
                            .short("m")
                            .help("Filter merge requests by scope")
                            .takes_value(true)
                            .possible_values(&["created_by_me", "assigned_to_me"])
                    )
                    .arg(
                        clap::Arg::with_name("labels")
                            .long("labels")
                            .short("l")
                            .help("Filter merge requests by label(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                            .conflicts_with_all(&["labelled", "unlabelled"])
                    )
                    .arg(
                        clap::Arg::with_name("unlabelled")
                            .long("unlabelled")
                            .help("Only return merge requests that have no labels")
                    )
                    .arg(
                        clap::Arg::with_name("labelled")
                            .long("labelled")
                            .help("Only return merge requests that have any label")
                    )
                    .arg(
                        clap::Arg::with_name("author")
                            .long("author")
                            .short("a")
                            .help("Filter merge requests by author username")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("approved_by")
                            .long("approved_by")
                            .help("Filter merge requests which are approved by username(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                            .conflicts_with_all(&["no_approvals", "any_approvals"])
                    )
                    .arg(
                        clap::Arg::with_name("no_approvals")
                            .long("no_approvals")
                            .help("Only return merge requests that have no approvals")
                    )
                    .arg(
                        clap::Arg::with_name("any_approvals")
                            .long("any_approvals")
                            .help("Only return merge requests that have at least one approval")
                    )
                    .arg(
                        clap::Arg::with_name("approvers")
                            .long("approvers")
                            .help("Filter merge requests which have username(s) as approver(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                            .conflicts_with_all(&["no_approver", "any_approvers"])
                    )
                    .arg(
                        clap::Arg::with_name("no_approvers")
                            .long("no_approver")
                            .help("Only return merge requests that have no approvers")
                    )
                    .arg(
                        clap::Arg::with_name("any_approvers")
                            .long("any_approvers")
                            .help("Only return merge requests that have at least one approver")
                    )
                    .arg(
                        clap::Arg::with_name("assignee")
                            .long("assignee")
                            .help("Filter merge requests which are assigned to a username")
                            .takes_value(true)
                            .empty_values(false)
                            .conflicts_with_all(&["assigned", "unassigned"])
                    )
                    .arg(
                        clap::Arg::with_name("unassigned")
                            .long("unassigned")
                            .help("Only return merge requests that are unassigned")
                    )
                    .arg(
                        clap::Arg::with_name("assigned")
                            .long("assigned")
                            .help("Only return merge requests that are assigned")
                    )
                    .arg(
                        clap::Arg::with_name("filter")
                            .long("filter")
                            .short("f")
                            .help("Filter merge requests by search string")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("created_after")
                            .long("created_after")
                            .short("c")
                            .help("Fetch merge requests created after a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("created_before")
                            .long("created_before")
                            .short("C")
                            .help("Fetch merge requests created before a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("updated_after")
                            .long("updated_after")
                            .short("u")
                            .help("Fetch merge requests updated after a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("updated_before")
                            .long("updated_before")
                            .short("U")
                            .help("Fetch merge requests updated before a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("wip")
                            .long("wip")
                            .short("w")
                            .help("Fetch merge requests which are works in progress")
                    )
                    .arg(
                        clap::Arg::with_name("order_by")
                            .long("order_by")
                            .short("o")
                            .help("Order results by given field")
                            .takes_value(true)
                            .possible_values(
                                &["created_on",
                                "updated_on",
                                ])
                            .default_value("created_on")
                    )
                    .arg(
                        clap::Arg::with_name("descending")
                            .long("desc")
                            .short("D")
                            .help("Sort results in descending order")
                    )
                    .arg(
                        clap::Arg::with_name("ascending")
                            .long("asc")
                            .short("A")
                            .help("Sort results in ascending order")
                    )
                    .arg(
                        clap::Arg::with_name("max")
                            .long("max")
                            .takes_value(true)
                            .empty_values(false)
                            .default_value("40")
                            .help("Maximum records to return")
                            .validator(validator::check_u32)
                    )
                    .after_help(
"Note that the `_before` and `_after` fields take a duration string similar to `12y 3months 3weeks \
9d 3hr 20sec`. You may use units of the long form: `years, months, days, weeks` etc, or the short \
form: `y, M, d, h, m, s`."
                    ),
            )
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
                clap::SubCommand::with_name("checkout")
                    .about("Checks out a merge request locally")
                    .setting(clap::AppSettings::ColoredHelp)
                    .visible_aliases(&["co"])
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge request ID to checkout")
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
                clap::SubCommand::with_name("show")
                    .about("Shows merge request information in the terminal")
                    .visible_aliases(&["info", "get"])
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Merge request ID to show")
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
            ("checkout", Some(a)) => checkout::checkout_merge_request_cmd(a.clone(), config, *gitlabclient)?,
            ("show", Some(a)) => show::show_mr_cmd(a.clone(), config, *gitlabclient)?,
            ("list", Some(a)) => list::list_mrs_cmd(a.clone(), config, *gitlabclient)?,
            ("close", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Close, config, *gitlabclient)?,
            ("reopen", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Reopen, config, *gitlabclient)?,
            ("lock", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Lock, config, *gitlabclient)?,
            ("unlock", Some(a)) => quick_edit::quick_edit_mr_cmd(a.clone(), ShortCmd::Unlock, config, *gitlabclient)?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
