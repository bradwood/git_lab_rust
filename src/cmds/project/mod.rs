mod attach;
mod create;
mod open;
mod show;

use anyhow::Result;
use anyhow::Context;

use crate::config;
use crate::gitlab;
use crate::subcommand;
use crate::utils::validator;


/// This implements the `project` command. It proves the ability to create, query and manipulate
/// projects in GitLab.
pub struct Project<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl subcommand::SubCommand for Project<'_> {
    fn gen_clap_command(&self) -> clap::App {
        let c = self.clap_cmd.clone();
        c.about("Creates, manipulates and queries projects")
            .setting(clap::AppSettings::ColoredHelp)
            .setting(clap::AppSettings::VersionlessSubcommands)
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("show")
                    .about("Shows project information in the terminal")
                    .visible_aliases(&["info", "get"])
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to view")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("open")
                    .about("Opens the project in the default browser")
                    .visible_aliases(&["view", "browse"])
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("url")
                            .short("u")
                            .long("print_url")
                            .help("Print the URL instead of opening it.")
                    )
                    .arg(
                        clap::Arg::with_name("id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to view")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
                    .after_help(
"This command will open the default browser to the URL of the attached project, or the project with \
the project_id if passed in. It will use the BROWSER environment variable to determine which  browser \
to use. If this is not set, on Linux, it will try `xdg-open(1)`",
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("attach")
                    .about("Attaches a GitLab project to a local repo")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("id")
                            .short("p")
                            .long("project_id")
                            .help("Project ID to attach")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_u64)
                    )
                    .after_help(
"Attaching to a project makes a permanent configuration change to the local repo using standard \
git-config(1) machinery to associate a GitLab project to a local repo. Subsequent commands that are \
invoked in a project context can then use the attached project's identifier when they are invoked.\
\n
If the project ID is passed it will be attached without verification against the GitLab server; \
if not, the command will try to infer which project to attach to by checking if a git remote is \
configured at the the GitLab host. If one is it will attempt to attach to it.\
\n
If invoked outside the context of a local repo, the command will fail.",
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("create")
                    .about("Creates a GitLab project")
                    .setting(clap::AppSettings::ColoredHelp)
                    .setting(clap::AppSettings::DeriveDisplayOrder)
                    .arg(
                        clap::Arg::with_name("name")
                            .help("Project name")
                            .takes_value(true)
                            .empty_values(false)
                            .required(true)
                    )
                    .arg(
                        clap::Arg::with_name("path")
                            .long("path")
                            .short("p")
                            .help("Project path/slug")
                            .empty_values(false)
                            .takes_value(true)
                            .validator(validator::check_project_slug)
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
                        clap::Arg::with_name("namespace_id")
                            .long("namespace_id")
                            .short("n")
                            .help("Project Namespace ID")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("default_branch")
                            .long("default_branch")
                            .short("b")
                            .help("Default branch")
                            // .default_value("master")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_branch_name)
                    )
                    .arg(
                        clap::Arg::with_name("import_url")
                            .long("import_url")
                            .short("u")
                            .help("Imports repository from URL")
                            .takes_value(true)
                            .empty_values(false)
                            .validator(validator::check_url)
                    )
                    .arg(
                        clap::Arg::with_name("merge_approval_count")
                            .long("merge_approval_count")
                            .takes_value(true)
                            .help("Sets how many merge request approvals are required before merge")
                            .empty_values(false)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("build_timeout")
                            .long("build_timeout")
                            .takes_value(true)
                            .help("Sets timeout before killing CI/CD pipeline in minutes")
                            .empty_values(false)
                            .validator(validator::check_u64)
                    )
                    .arg(
                        clap::Arg::with_name("build_coverage_regex")
                            .long("build_coverage_regex")
                            .help("Sets regex to use to extract coverage stats from pipelines")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("ci_config_path")
                            .long("ci_config_path")
                            .help("Sets path to gitlab ci config file.")
                            // .default_value(".gitlab-ci.yml")
                            .takes_value(true)
                            .empty_values(false)
                    )
                    .arg(
                        clap::Arg::with_name("visibility")
                            .long("visibility")
                            .short("v")
                            .takes_value(true)
                            .possible_values(&["public", "internal", "private"])
                    )
                    .arg(
                        clap::Arg::with_name("issues_access_level")
                            .long("issues_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("disable_issues")
                            .long("disable_issues")
                            .help("Deprecated - use `issues_access_level`")
                    )
                    .arg(
                        clap::Arg::with_name("forking_access_level")
                            .long("forking_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("repo_access_level")
                            .long("repo_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("mr_access_level")
                            .long("mr_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("disable_mr")
                            .long("disable_mr")
                            .help("Deprecated - use `mr_access_level`")
                    )
                    .arg(
                        clap::Arg::with_name("builds_access_level")
                            .long("builds_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("disable_builds")
                            .long("disable_builds")
                            .help("Deprecated - use `builds_access_level`")
                    )
                    .arg(
                        clap::Arg::with_name("wiki_access_level")
                            .long("wiki_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("disable_wiki")
                            .long("disable_wiki")
                            .help("Deprecated - use `wiki_access_level`")
                    )
                    .arg(
                        clap::Arg::with_name("snippets_access_level")
                            .long("snippets_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled"])
                    )
                    .arg(
                        clap::Arg::with_name("disable_snippets")
                            .long("disable_snippets")
                            .help("Deprecated - use `snippets_access_level`")
                    )
                    .arg(
                        clap::Arg::with_name("pages_access_level")
                            .long("pages_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                    )
                    .arg(
                        clap::Arg::with_name("disable_emails")
                            .long("disable_emails")
                            .help("Disable email alerts")
                    )
                    .arg(
                        clap::Arg::with_name("enable_container_registry")
                            .long("enable_container_registry")
                            .help("Enables the project's container registry")
                    )
                    .arg(
                        clap::Arg::with_name("enable_lfs")
                            .long("enable_lfs")
                            .help("Enables large file support")
                    )
                    .arg(
                        clap::Arg::with_name("enable_request_access")
                            .long("enable_request_access")
                            .help("Enables users to request member access")
                    )
                    .arg(
                        clap::Arg::with_name("print_merge_request_url")
                            .long("print_merge_request_url")
                            .help("Prints merge request URL on command line when pushing")
                    )
                    .arg(
                        clap::Arg::with_name("auto_devops_deploy_strategy")
                            .long("auto_devops_deploy_strategy")
                            .takes_value(true)
                            .possible_values(&["continuous", "manual", "timed_incremental"])
                            .requires("enable_auto_devops")
                    )
                    .arg(
                        clap::Arg::with_name("enable_auto_devops")
                            .long("enable_auto_devops")
                            .help("Enables auto-devops feature")
                    )
                    .arg(
                        clap::Arg::with_name("remove_source_branch_after_merge")
                            .long("remove_source_branch_after_merge")
                            .help("Deletes branch after it is merged")
                    )
                    .arg(
                        clap::Arg::with_name("enable_shared_runners")
                            .long("enable_shared_runners")
                            .help("Enables shared CI/CD runners")
                    )
                    .arg(
                        clap::Arg::with_name("tags")
                            .long("tags")
                            .short("t")
                            .help("Sets tag list for the project")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    // .arg(
                    //     clap::Arg::with_name("container_expiration_policy")
                    //         .long("container_expiration_policy")
                    //         .help("Sets container expiration policy attributes. See GitLab docs for details.")
                    //         .takes_value(true)
                    //         .multiple(true)
                    //         .requires("enable_container_registry")
                    //         .empty_values(false)
                    //         .require_delimiter(true)
                    // )
                    .arg(
                        clap::Arg::with_name("enable_public_builds")
                            .long("enable_public_builds")
                            .help("Makes builds publically viewable")
                    )
                    .arg(
                        clap::Arg::with_name("resolve_old_discussions")
                            .long("resolve_old_discussions")
                            .help("Enables automatic resolution of outdated diff discussions")
                    )
                    .arg(
                        clap::Arg::with_name("only_merge_on_good_ci")
                            .long("only_merge_on_good_ci")
                            .help("Ensures that merges only occur if pipeline succeeds")
                    )
                    .arg(
                        clap::Arg::with_name("only_merge_on_closed_discussions")
                            .long("only_merge_on_closed_discussions")
                            .help("Ensures that merges only occur once discussions are resolved")
                    )
                    .arg(
                        clap::Arg::with_name("auto_close_referenced_issues")
                            .long("auto_close_referenced_issues")
                            .help("Enables the automatic closure of related issues on successful merge requests")
                    )
                    .arg(
                        clap::Arg::with_name("auto_cancel_pending_pipelines")
                            .long("auto_cancel_pending_pipelines")
                            .help("Enables the automatic cancellation of pipelines that are superseded by newer ones")
                    )
                    .arg(
                        clap::Arg::with_name("enable_packages")
                            .long("enable_packages")
                            .help("Enables packages feature in project")
                    )
                    .arg(
                        clap::Arg::with_name("initialise_with_readme")
                            .long("initialise_with_readme")
                            .help("Creates an empty README.md")
                    )
                    .arg(
                        clap::Arg::with_name("enable_mirror")
                            .long("enable_mirror")
                            .help("Enables pull mirroring for the project")
                    )
                    .arg(
                        clap::Arg::with_name("mirror_triggers_builds")
                            .long("mirror_triggers_builds")
                            .help("Enables builds when mirroring occurs")
                            .requires("enable_mirror")
                    )
                    .arg(
                        clap::Arg::with_name("merge_method")
                            .long("merge_method")
                            .short("m")
                            .takes_value(true)
                            .empty_values(false)
                            .possible_values(&["merge", "rebase-merge", "fast-forward"])
                    )
                    .arg(
                        clap::Arg::with_name("pipeline_git_strategy")
                            .long("pipeline_git_strategy")
                            .takes_value(true)
                            .empty_values(false)
                            .possible_values(&["fetch", "clone"])
                    )
                    .after_help(
"Note that the `*_access_level` are enhancements for the various `disable_*` flags which are  \
due to be deprecated at some point. However, at the time of writing, there is a GitLab bug which \
means that passing `disabled` to the `*_access_level` switches doesn't have any effect. So the \
deprecated `disable_*` flags (which do _currently_ work) remain in place for now. \
If you have errors using the `*_disabled` flags your GitLab server may no longer support them.",
                    ),
            )
    }

    fn run(&self, config: config::Config, args: clap::ArgMatches) -> Result<()> {

        trace!("Config: {:?}", config);
        debug!("Args: {:#?}", args);

        let gitlabclient = gitlab::new(&config).context("Could not create GitLab client connection.")?;

        match args.subcommand() {
            ("create", Some(a)) => create::create_project_cmd(a.clone(), config, *gitlabclient)?,
            ("attach", Some(a)) => attach::attach_project_cmd(a.clone(), config, *gitlabclient)?,
            ("open", Some(a)) => open::open_project_cmd(a.clone(), config, *gitlabclient)?,
            ("show", Some(a)) => show::show_project_cmd(a.clone(), config, *gitlabclient)?,
            _ => unreachable!(),
        }

        Ok(())
    }
}
