// mod create;
mod open;
mod show;
mod create;

use anyhow::Result;
use anyhow::Context;

use crate::config;
use crate::gitlab;
use crate::subcommand;
use crate::utils::validator;


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
                    //TODO: make this work with @usernames
                    .arg(
                        clap::Arg::with_name("assignees")
                            .long("assignees")
                            .short("a")
                            .help("Sets issue assignee(s) IDs")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                            .validator(validator::check_u64)
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
            _ => unreachable!(),
        }

        Ok(())
    }
}
