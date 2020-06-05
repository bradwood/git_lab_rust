use std::borrow::Cow;

use anyhow::{anyhow, Context, Result};
use clap::value_t_or_exit;
use serde::Deserialize;

use crate::gitlab::converter::{
    auto_devops_deploy_strategy_from_str, enable_state_from_str, feature_access_level_from_str,
    feature_access_level_public_from_str, merge_method_from_str, pipeline_git_strategy_from_str,
    visibility_level_from_str,
};
use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::{api, Client, CreateProject, CreateProjectBuilder, Query};

#[derive(Debug, Deserialize)]
struct Project {
    id: u64,
    web_url: String,
}

pub fn generate_project_builder<'a>(
    args: &'a clap::ArgMatches,
    p: &'a mut CreateProjectBuilder<'a>,
) -> CreateProject<'a> {
    for arg in &args.args {
        let (key, _) = arg;
        match *key {
            // url argument -- validation done by clap.rs
            "import_url" => p.import_url(Cow::from(args.value_of("import_url").unwrap())),

            // u64 arguments
            "namespace_id" => p.namespace_id(value_t_or_exit!(args, "namespace_id", u64)),
            "merge_approval_count" => {
                p.approvals_before_merge(value_t_or_exit!(args, "merge_approval_count", u64))
            }
            "build_timeout" => p.build_timeout(value_t_or_exit!(args, "build_timeout", u64)),

            // basic boolean flags
            "enable_lfs" => p.lfs_enabled(true),
            "enable_request_access" => p.request_access_enabled(true),
            "enable_container_registry" => p.container_registry_enabled(true),
            "print_merge_request_url" => p.printing_merge_request_link_enabled(true),
            "enable_auto_devops" => p.auto_devops_enabled(true),
            "enable_shared_runners" => p.shared_runners_enabled(true),
            "remove_source_branch_after_merge" => p.remove_source_branch_after_merge(true),
            "enable_public_builds" => p.public_builds(true),
            "resolve_old_discussions" => p.resolve_outdated_diff_discussions(true),
            "only_merge_on_good_ci" => p.only_allow_merge_if_pipeline_succeeds(true),
            "only_merge_on_closed_discussions" => {
                p.only_allow_merge_if_all_discussions_are_resolved(true)
            }
            "auto_close_referenced_issues" => p.autoclose_referenced_issues(true),
            "disable_emails" => p.emails_disabled(true),
            "enable_packages" => p.packages_enabled(true),
            "enable_mirror" => p.mirror(true),
            "mirror_triggers_builds" => p.mirror_trigger_builds(true),
            "initialise_with_readme" => p.initialize_with_readme(true),

            // deprecated boolean disable flags
            "disable_issues" => p.issues_enabled(false),
            "disable_mr" => p.merge_requests_enabled(false),
            "disable_builds" => p.jobs_enabled(false),
            "disable_wiki" => p.wiki_enabled(false),
            "disable_snippets" => p.snippets_enabled(false),

            // straight string
            "description" => p.description(args.value_of("description").unwrap()),
            "default_branch" => p.default_branch(args.value_of("default_branch").unwrap()),
            "build_coverage_regex" => {
                p.build_coverage_regex(args.value_of("build_coverage_regex").unwrap())
            }
            "ci_config_path" => p.ci_config_path(args.value_of("ci_config_path").unwrap()),

            // EnableState enum from boolean
            "auto_cancel_pending_pipelines" => {
                p.auto_cancel_pending_pipelines(enable_state_from_str("enabled").unwrap())
            }

            //  specific conversion to auto_devops_deploy_strategy enum
            "auto_devops_deploy_strategy" => p.auto_devops_deploy_strategy(
                auto_devops_deploy_strategy_from_str(
                    args.value_of("auto_devops_deploy_strategy").unwrap(),
                )
                .unwrap(),
            ),

            //  specific conversion to visibilily_level enum
            "visibility" => p.visibility(
                visibility_level_from_str(args.value_of("visibility").unwrap()).unwrap(),
            ),

            // specific conversion to feature_access_level enum
            // NOTE: API for these is currently buggy. See https://gitlab.com/gitlab-org/gitlab/-/issues/219482
            "repo_access_level" => p.repository_access_level(
                feature_access_level_from_str(args.value_of("repo_access_level").unwrap()).unwrap(),
            ),
            "issues_access_level" => p.issues_access_level(
                feature_access_level_from_str(args.value_of("issues_access_level").unwrap())
                    .unwrap(),
            ),
            "forking_access_level" => p.forking_access_level(
                feature_access_level_from_str(args.value_of("forking_access_level").unwrap())
                    .unwrap(),
            ),
            "mr_access_level" => p.merge_requests_access_level(
                feature_access_level_from_str(args.value_of("mr_access_level").unwrap()).unwrap(),
            ),
            "builds_access_level" => p.builds_access_level(
                feature_access_level_from_str(args.value_of("builds_access_level").unwrap())
                    .unwrap(),
            ),
            "wiki_access_level" => p.wiki_access_level(
                feature_access_level_from_str(args.value_of("wiki_access_level").unwrap()).unwrap(),
            ),
            "snippets_access_level" => p.snippets_access_level(
                feature_access_level_from_str(args.value_of("snippets_access_level").unwrap())
                    .unwrap(),
            ),

            // specific conversion to feature_access_level_public enum
            "pages_access_level" => p.pages_access_level(
                feature_access_level_public_from_str(args.value_of("pages_access_level").unwrap())
                    .unwrap(),
            ),

            // specific conversion to merge_method enum
            "merge_method" => p.merge_method(
                merge_method_from_str(args.value_of("merge_method").unwrap()).unwrap(),
            ),

            // specific conversion to build_git_strategy enum
            "pipeline_git_strategy" => p.build_git_strategy(
                pipeline_git_strategy_from_str(args.value_of("pipeline_git_strategy").unwrap())
                    .unwrap(),
            ),

            // list of tags
            "tags" => p.tags(args.values_of("tags").unwrap()),

            // project name -- mandated by clap.rs
            "name" => p.name(args.value_of("name").unwrap()),

            _ => unreachable!(),
        };
    }
    p.build().unwrap()
}

pub fn create_project_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = CreateProject::builder();
    let endpoint = generate_project_builder(&args, &mut p);

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json  = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Fail")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        },

        Some(OutputFormat::Text) => {
            let project: Project = endpoint
                .query(&gitlabclient)
                .context("Failed to create project - check for name or path clashes on the server")?;

            println!("Project id: {}", project.id);
            println!("Project URL: {}", project.web_url);
            Ok(())
        },
        _ => Err(anyhow!("Bad output format in config")),
    }
}

#[cfg(test)]
mod project_create_unit_tests {
    use crate::cmds::project;
    use crate::subcommand::SubCommand;
    use clap::SubCommand as ClapSubCommand;

    use super::*;

    #[test]
    fn test_generate_project_builder() {
        // GIVEN
        let mut p = CreateProject::builder();

        let p_cmd = project::Project {
            clap_cmd: ClapSubCommand::with_name("project"),
        };

        let args = p_cmd.gen_clap_command().get_matches_from(vec![
            "project",
            "create",
            "project_name",
            "--namespace_id",
            "3",
            "--default_branch",
            "branch_name",
            "-d",
            "description text",
            "--issues_access_level",
            "disabled",
            "--repo_access_level",
            "disabled",
            "--forking_access_level",
            "disabled",
            "--builds_access_level",
            "disabled",
            "--mr_access_level",
            "disabled",
            "--wiki_access_level",
            "disabled",
            "--snippets_access_level",
            "disabled",
            "--pages_access_level",
            "public",
            "--disable_emails",
            "--resolve_old_discussions",
            "--enable_container_registry",
            "--enable_shared_runners",
            "-v",
            "private",
            // "--import_url", "https://gitub.com/blah/blah.git", THIS NEEDS SEPARATE TESTING
            "--enable_public_builds",
            "--only_merge_on_good_ci",
            "--only_merge_on_closed_discussions",
            "--merge_method",
            "fast-forward",
            "--auto_close_referenced_issues",
            "--remove_source_branch_after_merge",
            "--enable_lfs",
            "--enable_request_access",
            "--tags",
            "one,two",
            "--print_merge_request_url",
            "--pipeline_git_strategy",
            "clone",
            "--build_timeout",
            "54",
            "--auto_cancel_pending_pipelines",
            "--build_coverage_regex",
            ".*",
            "--ci_config_path",
            "filename",
            "--enable_auto_devops",
            "--auto_devops_deploy_strategy",
            "timed_incremental",
            "--merge_approval_count",
            "3",
            "--enable_mirror",
            "--mirror_triggers_builds",
            "--initialise_with_readme",
            "--enable_packages",
            "--disable_issues",
            "--disable_mr",
            "--disable_builds",
            "--disable_snippets",
            "--disable_wiki",
        ]);
        let matches = args.subcommand_matches("create");

        // WHEN
        let endpoint = generate_project_builder(&matches.unwrap(), &mut p);

        // THEN
        let endpoint_debug = r###"CreateProject {
    name_and_path: Name {
        name: "project_name",
    },
    namespace_id: Some(
        3,
    ),
    default_branch: Some(
        "branch_name",
    ),
    description: Some(
        "description text",
    ),
    issues_access_level: Some(
        Disabled,
    ),
    repository_access_level: Some(
        Disabled,
    ),
    merge_requests_access_level: Some(
        Disabled,
    ),
    forking_access_level: Some(
        Disabled,
    ),
    builds_access_level: Some(
        Disabled,
    ),
    wiki_access_level: Some(
        Disabled,
    ),
    snippets_access_level: Some(
        Disabled,
    ),
    pages_access_level: Some(
        Public,
    ),
    emails_disabled: Some(
        true,
    ),
    resolve_outdated_diff_discussions: Some(
        true,
    ),
    container_registry_enabled: Some(
        true,
    ),
    container_expiration_policy_attributes: None,
    shared_runners_enabled: Some(
        true,
    ),
    visibility: Some(
        Private,
    ),
    import_url: None,
    public_builds: Some(
        true,
    ),
    only_allow_merge_if_pipeline_succeeds: Some(
        true,
    ),
    only_allow_merge_if_all_discussions_are_resolved: Some(
        true,
    ),
    merge_method: Some(
        FastForward,
    ),
    autoclose_referenced_issues: Some(
        true,
    ),
    remove_source_branch_after_merge: Some(
        true,
    ),
    lfs_enabled: Some(
        true,
    ),
    request_access_enabled: Some(
        true,
    ),
    tag_list: {
        "one",
        "two",
    },
    printing_merge_request_link_enabled: Some(
        true,
    ),
    build_git_strategy: Some(
        Clone,
    ),
    build_timeout: Some(
        54,
    ),
    auto_cancel_pending_pipelines: Some(
        Enabled,
    ),
    build_coverage_regex: Some(
        ".*",
    ),
    ci_config_path: Some(
        "filename",
    ),
    auto_devops_enabled: Some(
        true,
    ),
    auto_devops_deploy_strategy: Some(
        TimedIncremental,
    ),
    repository_storage: None,
    approvals_before_merge: Some(
        3,
    ),
    external_authorization_classification_label: None,
    mirror: Some(
        true,
    ),
    mirror_trigger_builds: Some(
        true,
    ),
    initialize_with_readme: Some(
        true,
    ),
    template_name: None,
    template_project_id: None,
    use_custom_template: None,
    group_with_project_templates_id: None,
    packages_enabled: Some(
        true,
    ),
    issues_enabled: Some(
        false,
    ),
    merge_requests_enabled: Some(
        false,
    ),
    jobs_enabled: Some(
        false,
    ),
    wiki_enabled: Some(
        false,
    ),
    snippets_enabled: Some(
        false,
    ),
}"###;

        assert_eq!(endpoint_debug, format!("{:#?}", endpoint))
    }
}
