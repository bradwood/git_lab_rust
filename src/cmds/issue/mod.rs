mod create;
mod list;
mod open;
mod show;
mod quick_edit;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc, NaiveDate};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::config;
use crate::gitlab::Issue as GLIssue;
use crate::gitlab::IssueBuilder;
use crate::gitlab;
use crate::subcommand;
use crate::utils::validator;
use crate::utils;

#[derive(Debug)]
pub enum ShortCmd {
    Close,
    Reopen,
    Lock,
    Unlock,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
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
    merge_requests_count: u64,
    upvotes: u64,
    downvotes: u64,
    due_date: Option<NaiveDate>,
    confidential: bool,
    discussion_locked: Option<bool>,
    web_url: String,
    task_completion_status: Option<Map<String, Value>>,
    weight: Option<u64>,
    has_tasks: Option<bool>,
    task_status: Option<String>,
    references: Map<String, Value>,
    subscribed: Option<bool>,
}

pub fn generate_basic_issue_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    i: &'a mut IssueBuilder<'a>,
) -> Result<GLIssue<'a>> {

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    i.project(project_id);
    i.issue(args.value_of("id").unwrap().parse::<u64>().unwrap());
    i.build()
        .map_err(|e| anyhow!("Could not construct query for this issue.\n {}",e))
}

/// This implements the `issue` command. It proves the ability to create, query and manipulate
/// issues in GitLab.
pub struct IssueCmd<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl subcommand::SubCommand for IssueCmd<'_> {
    fn gen_clap_command(&self) -> clap::App {
        let c = self.clap_cmd.clone();
        c.about("Creates, manipulates and queries issues")
            .setting(clap::AppSettings::ColoredHelp)
            .setting(clap::AppSettings::VersionlessSubcommands)
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("list")
                    .about("Lists issues")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("state")
                            .long("state")
                            .short("s")
                            .help("Filter issues by state")
                            .takes_value(true)
                            .possible_values(&["all", "closed", "opened"])
                            .default_value("opened")
                    )
                    .arg(
                        clap::Arg::with_name("scope")
                            .long("scope")
                            .short("m")
                            .help("Filter issues by scope")
                            .takes_value(true)
                            .possible_values(&["created_by_me", "assigned_to_me"])
                    )
                    .arg(
                        clap::Arg::with_name("labels")
                            .long("labels")
                            .short("l")
                            .help("Filter issues by label(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                            .conflicts_with_all(&["labelled", "unlabelled"])
                    )
                    .arg(
                        clap::Arg::with_name("unlabelled")
                            .long("unlabelled")
                            .help("Only return issues that have no labels")
                    )
                    .arg(
                        clap::Arg::with_name("labelled")
                            .long("labelled")
                            .help("Only return issues that have any label")
                    )
                    .arg(
                        clap::Arg::with_name("author")
                            .long("author")
                            .short("a")
                            .help("Filter issues by author username")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("assignees")
                            .long("assignees")
                            .help("Filter issues which are assigned to as set of usernames")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                            .conflicts_with_all(&["assigned", "unassigned"])
                    )
                    .arg(
                        clap::Arg::with_name("unassigned")
                            .long("unassigned")
                            .help("Only return issues that are unassigned")
                    )
                    .arg(
                        clap::Arg::with_name("assigned")
                            .long("assigned")
                            .help("Only return issues that are assigned")
                    )
                    .arg(
                        clap::Arg::with_name("weight")
                            .long("weight")
                            .short("w")
                            .help("Filter issues by weight")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_u32)
                            .conflicts_with_all(&["weighted", "unweighted"])
                    )
                    .arg(
                        clap::Arg::with_name("weighted")
                            .help("Only return issues that have a weight")
                            .long("weighted")
                    )
                    .arg(
                        clap::Arg::with_name("unweighted")
                            .help("Only return issues that have no weight")
                            .long("unweighted")
                    )
                    .arg(
                        clap::Arg::with_name("filter")
                            .long("filter")
                            .short("f")
                            .help("Filter issues by search string")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("created_after")
                            .long("created_after")
                            .short("c")
                            .help("Fetch issues created after a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("created_before")
                            .long("created_before")
                            .short("C")
                            .help("Fetch issues created before a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("updated_after")
                            .long("updated_after")
                            .short("u")
                            .help("Fetch issues updated after a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("updated_before")
                            .long("updated_before")
                            .short("U")
                            .help("Fetch issues updated before a certain time period")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_valid_humantime_duration)
                    )
                    .arg(
                        clap::Arg::with_name("confidential")
                            .long("confidential")
                            .short("p")
                            .help("Fetch only confidential issues")
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
                                "priority",
                                "due_date",
                                "relative_position",
                                "label_priority",
                                "milestone_date",
                                "popularity",
                                "weight",
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
                clap::SubCommand::with_name("status")
                    .about("Shows issues related to you")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for issues in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("unlock")
                    .about("Unlocks an issue")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Issue ID to unlock")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for issue in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("lock")
                    .about("Locks an issue")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Issue ID to lock")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for issue in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("reopen")
                    .about("Re-opens an issue")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Issue ID to re-open")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for issue in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("close")
                    .about("Closes an issue")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Issue ID to close")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for issue in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("show")
                    .about("Shows issue information in the terminal")
                    .visible_aliases(&["info", "get"])
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Issue ID to show")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("project_id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to look for issue in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
            )
            .subcommand(
                clap::SubCommand::with_name("open")
                    .about("Opens the issue in the default browser")
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
                            .help("Project ID to look for issue in. Defaults to attached Project ID.")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("id")
                            .help("Issue ID to open")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                            .validator(validator::check_u64)
                    )
                    .after_help(
"This command will open the default browser to the URL of the passed issue. It will use the BROWSER \
environment variable to determine which browser to use. If this is not set, on Linux, it will \
try `xdg-open(1)`.",
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("create")
                    .about("Creates a GitLab issue")
                    .setting(clap::AppSettings::ColoredHelp)
                    .setting(clap::AppSettings::DeriveDisplayOrder)
                    .arg(
                        clap::Arg::with_name("title")
                            .help("Issue title")
                            .takes_value(true)
                            .empty_values(false)
                    )
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
                        clap::Arg::with_name("description")
                            .long("desc")
                            .short("d")
                            .help("Project description")
                            .empty_values(false)
                            .takes_value(true)
                    )
                    .arg(
                        clap::Arg::with_name("confidential")
                        .long("confidential")
                        .short("c")
                        .help("Sets the issue to be confidential")
                    )
                    .arg(
                        clap::Arg::with_name("milestone_id")
                            .long("milestone_id")
                            .short("m")
                            .takes_value(true)
                            .help("Associates the issue to a milestone")
                            .empty_values(false)
                            .validator(validator::check_u32)
                    )
                    .arg(
                        clap::Arg::with_name("due_date")
                            .long("due_date")
                            .short("u")
                            .takes_value(true)
                            .help("Due date in format YYYY-MM-DD")
                            .empty_values(false)
                            .validator(validator::check_yyyy_mm_dd)
                    )
                    .arg(
                        clap::Arg::with_name("weight")
                            .long("weight")
                            .short("w")
                            .takes_value(true)
                            .help("Sets the weight of the issue")
                            .empty_values(false)
                            .validator(validator::check_u32)
                    )
                    .arg(
                        clap::Arg::with_name("labels")
                            .long("labels")
                            .short("l")
                            .help("Sets issue label(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    .arg(
                        clap::Arg::with_name("assignees")
                            .long("assignees")
                            .short("a")
                            .help("Username(s) of issue assignee(s)")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    .after_help(
"If the title is is omitted then the user will be prompted for issue parameters interactively",
                    ),
            )
    }

    fn run(&self, config: config::Config, args: clap::ArgMatches) -> Result<()> {

        trace!("Config: {:?}", config);
        debug!("Args: {:#?}", args);

        let gitlabclient = gitlab::new(&config).context("Could not create GitLab client connection.")?;

        match args.subcommand() {
            ("create", Some(a)) => create::create_issue_cmd(a.clone(), config, *gitlabclient)?,
            ("open", Some(a)) => open::open_issue_cmd(a.clone(), config, *gitlabclient)?,
            ("show", Some(a)) => show::show_issue_cmd(a.clone(), config, *gitlabclient)?,
            ("list", Some(a)) => list::list_issues_cmd(a.clone(), config, *gitlabclient)?,
            // ("status", Some(a)) => status::status_issues_cmd(a.clone(), config, *gitlabclient)?,
            ("close", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Close, config, *gitlabclient)?,
            ("reopen", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Reopen, config, *gitlabclient)?,
            ("lock", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Lock, config, *gitlabclient)?,
            ("unlock", Some(a)) => quick_edit::quick_edit_issue_cmd(a.clone(), ShortCmd::Unlock, config, *gitlabclient)?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
