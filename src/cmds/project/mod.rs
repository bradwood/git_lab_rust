mod create;

use anyhow::Result;

use crate::config;
use crate::gitlab;
use crate::gitlab::IfGitLabNew;
use crate::subcommand;


/// This implements the `project` command. It proves the ability to create, query and manipulate
/// projects in GitLab.
///
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
                clap::SubCommand::with_name("attach")
                    .about("Attaches a GitLab project to a local repo")
                    .setting(clap::AppSettings::ColoredHelp)
                    .arg(
                        clap::Arg::with_name("name")
                            // .short("n")
                            .long("name")
                            .help("Project name to attach")
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::with_name("id")
                            // .short("p")
                            .long("project_id")
                            .help("Project ID to attach")
                            .takes_value(true),
                    )
                    .after_help(
"Attaching to a project makes a permanent configuration change to the local repo using standard \
git-config(1) machinery to associate a GitLab project to a local repo. Subsequent commands that are \
invoked in a project context will then use the attached project's identifier when they are invoked.\
\n
If neither the project id nor name is passed, this command will attempt to infer which project to \
attach to by checking if a git remote is configured at the the GitLab host. If one is, it will \
attempt to attach to it.\

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
                            .help("Project path/slug")
                            .empty_values(false)
                            .takes_value(true)
                    )
                    .arg(
                        clap::Arg::with_name("description")
                            .long("desc")
                            .help("Project description")
                            .empty_values(false)
                            .takes_value(true)
                    )
                    .arg(
                        clap::Arg::with_name("namespace_id")
                            .long("namespace_id")
                            .help("Project Namespace ID")
                            .takes_value(true)
                            .empty_values(false)
                            //TODO add validator
                    )
                    .arg(
                        clap::Arg::with_name("default_branch")
                            .long("default_branch")
                            .help("Default branch")
                            // .default_value("master")
                            .takes_value(true)
                            .empty_values(false)
                            //TODO add validator
                    )
                    .arg(
                        clap::Arg::with_name("import_url")
                            .long("import_url")
                            .help("Imports repository from URL")
                            .takes_value(true)
                            .empty_values(false)
                            //TODO add validator
                    )
                    .arg(
                        clap::Arg::with_name("build_timeout")
                            .long("build_timeout")
                            .takes_value(true)
                            .help("Sets timeout before killing CI/CD pipeline in minutes")
                            .empty_values(false)
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
                            //TODO add validator
                    )
                    .arg(
                        clap::Arg::with_name("visibility")
                            .long("visibility")
                            .takes_value(true)
                            .possible_values(&["public", "internal", "private"])
                            // .default_value("public")
                    )
                    .arg(
                        clap::Arg::with_name("repo_access_level")
                            .long("repo_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                            // .default_value("enabled")
                    )
                    .arg(
                        clap::Arg::with_name("mr_access_level")
                            .long("mr_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                            // .default_value("enabled")
                    )
                    .arg(
                        clap::Arg::with_name("builds_access_level")
                            .long("builds_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                            // .default_value("enabled")
                    )
                    .arg(
                        clap::Arg::with_name("wiki_access_level")
                            .long("wiki_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                            // .default_value("enabled")
                    )
                    .arg(
                        clap::Arg::with_name("snippets_access_level")
                            .long("snippets_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                            // .default_value("enabled")
                    )
                    .arg(
                        clap::Arg::with_name("pages_access_level")
                            .long("pages_access_level")
                            .takes_value(true)
                            .possible_values(&["disabled", "private", "enabled", "public"])
                            // .default_value("enabled")
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
                            .empty_values(false)
                            .possible_values(&["continuous", "manual", "timed_incremental"])
                            .requires("auto_devops_enabled")
                    )
                    .arg(
                        clap::Arg::with_name("enable_auto_devops")
                            .long("enable_auto_devops")
                            .help("Enables auto-devops feature")
                    )
                    .arg(
                        clap::Arg::with_name("enable_shared_runners")
                            .long("enable_shared_runners")
                            .help("Enables shared CI/CD runners")
                    )
                    .arg(
                        clap::Arg::with_name("tag_list")
                            .long("tag_list")
                            .help("Sets tag list for the project")
                            .takes_value(true)
                            .multiple(true)
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    .arg(
                        clap::Arg::with_name("container_expiration_policy")
                            .long("container_expiration_policy")
                            .help("Sets container expiration policy attributes. See GitLab docs for details.")
                            .takes_value(true)
                            .multiple(true)
                            .requires("enable_container_registry")
                            .empty_values(false)
                            .require_delimiter(true)
                    )
                    .arg(
                        clap::Arg::with_name("public_builds")
                            .long("public_builds")
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
                        clap::Arg::with_name("merge_method")
                            .long("merge_method")
                            .takes_value(true)
                            .empty_values(false)
                            .possible_values(&["merge", "rebase-merge", "fast-forward"])
                            // .default_value("merge")
                    )
                    .arg(
                        clap::Arg::with_name("pipeline_git_strategy")
                            .long("pipeline_git_strategy")
                            .takes_value(true)
                            .empty_values(false)
                            .possible_values(&["fetch", "clone"])
                            // .default_value("fetch")
                    )
            )
    }

    // How to test? FIXME Use a mock over IfGitLabNew?
    fn run(&self, config: config::Config, args: clap::ArgMatches) -> Result<()> {

        trace!("Config: {:?}", config);
        debug!("Args: {:#?}", args);

        let gitlab = *gitlab::GitLab::new(&config)?;

        match args.subcommand() {
            ("create", Some(create_args)) => create::create_project_cmd(create_args.clone(), gitlab)?,
            _ => ()
        }

        Ok(())
    }
}
