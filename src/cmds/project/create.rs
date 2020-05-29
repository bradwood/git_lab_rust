use std::borrow::Cow;

use anyhow::{Context, Result};
use clap::{value_t_or_exit, values_t_or_exit};
use serde::Deserialize;

use crate::gitlab::{
    CreateProject,
    Client,
    Query,
};
use crate::gitlab::converter::{
    auto_devops_deploy_strategy_from_str,
    enable_state_from_str,
    feature_access_level_from_str,
    feature_access_level_public_from_str,
    merge_method_from_str,
    pipeline_git_strategy_from_str,
    visibility_level_from_str,
};

#[derive(Debug, Deserialize)]
struct Project {
    id: u64,
    web_url: String,
}

pub fn create_project_cmd(
    args: clap::ArgMatches,
    gitlabclient: Client,
) -> Result<()> {

    let mut p = CreateProject::builder();

    // url argument -- validation done by clap.rs
    if args.occurrences_of("import_url") > 0 {
        p.import_url(Cow::from(args.value_of("import_url").unwrap()));
    }

    // u64 arguments
    if args.occurrences_of("namespace_id") > 0 {
        p.namespace_id(value_t_or_exit!(args, "namespace_id", u64));
    }
    if args.occurrences_of("build_timeout") > 0 {
        p.build_timeout(value_t_or_exit!(args, "build_timeout", u64));
    }

    // basic boolean flags
    if args.occurrences_of("enable_lfs") > 0 {
        p.lfs_enabled(true);
    }
    if args.occurrences_of("enable_request_access") > 0 {
        p.request_access_enabled(true);
    }
    if args.occurrences_of("enable_container_registry") > 0 {
        p.container_registry_enabled(true);
    }
    if args.occurrences_of("print_merge_request_url") > 0 {
        p.printing_merge_request_link_enabled(true);
    }
    if args.occurrences_of("enable_auto_devops") > 0 {
        p.auto_devops_enabled(true);
    }
    if args.occurrences_of("enable_shared_runners") > 0 {
        p.shared_runners_enabled(true);
    }
    if args.occurrences_of("enable_public_builds") > 0 {
        p.public_builds(true);
    }
    if args.occurrences_of("resolve_old_discussions") > 0 {
        p.resolve_outdated_diff_discussions(true);
    }
    if args.occurrences_of("only_merge_on_good_ci") > 0 {
        p.only_allow_merge_if_pipeline_succeeds(true);
    }
    if args.occurrences_of("only_merge_on_closed_discussions") > 0 {
        p.only_allow_merge_if_all_discussions_are_resolved(true);
    }
    if args.occurrences_of("auto_close_referenced_issues") > 0 {
        p.autoclose_referenced_issues(true);
    }

    // deprecated boolean disable flags
    if args.occurrences_of("disable_issues") > 0 {
        p.issues_enabled(false);
    }
    if args.occurrences_of("disable_mr") > 0 {
        p.merge_requests_enabled(false);
    }
    if args.occurrences_of("disable_builds") > 0 {
        p.jobs_enabled(false);
    }
    if args.occurrences_of("disable_wiki") > 0 {
        p.wiki_enabled(false);
    }
    if args.occurrences_of("disable_snippets") > 0 {
        p.snippets_enabled(false);
    }

    // straight string
    if args.occurrences_of("description") > 0 {
        p.description(args.value_of("description").unwrap());
    }
    if args.occurrences_of("default_branch") > 0 {
        p.default_branch(args.value_of("default_branch").unwrap());
    }
    if args.occurrences_of("build_coverage_regex") > 0 {
        p.build_coverage_regex(args.value_of("build_coverage_regex").unwrap());
    }
    if args.occurrences_of("ci_config_path") > 0 {
        p.ci_config_path(args.value_of("ci_config_path").unwrap());
    }

    // EnableState enum from boolean
    if args.occurrences_of("auto_cancel_pending_pipelines") > 0 {
        p.auto_cancel_pending_pipelines(enable_state_from_str(args.value_of("auto_cancel_pending_pipelines").unwrap()).unwrap());
    }

    //  specific conversion to auto_devops_deploy_strategy enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("auto_devops_deploy_strategy") > 0 {
        p.auto_devops_deploy_strategy(auto_devops_deploy_strategy_from_str(args.value_of("auto_devops_deploy_strategy").unwrap()).unwrap());
    }
    //  specific conversion to visibilily_level enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("visibility") > 0 {
        p.visibility(visibility_level_from_str(args.value_of("visibility").unwrap()).unwrap());
    }

    // specific conversion to feature_access_level enum - unwrap()'s are safe as problems will be
    // caught by clap.rs
    // NOTE: API for these is currently buggy. See https://gitlab.com/gitlab-org/gitlab/-/issues/219482
    if args.occurrences_of("repo_access_level") > 0 {
        p.repository_access_level(
            feature_access_level_from_str(args.value_of("repo_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("issues_access_level") > 0 {
        p.issues_access_level(
            feature_access_level_from_str(args.value_of("issues_access_level").unwrap()) .unwrap(),
        );
    }
    if args.occurrences_of("forking_access_level") > 0 {
        p.forking_access_level(
            feature_access_level_from_str(args.value_of("forking_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("mr_access_level") > 0 {
        p.merge_requests_access_level(
            feature_access_level_from_str(args.value_of("mr_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("builds_access_level") > 0 {
        p.builds_access_level(
            feature_access_level_from_str(args.value_of("builds_access_level").unwrap())
                .unwrap(),
        );
    }
    if args.occurrences_of("wiki_access_level") > 0 {
        p.wiki_access_level(
            feature_access_level_from_str(args.value_of("wiki_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("snippets_access_level") > 0 {
        p.snippets_access_level(
            feature_access_level_from_str(args.value_of("snippets_access_level").unwrap())
                .unwrap(),
        );
    }

    // specific conversion to feature_access_level_public enum - unwrap()'s are safe as problems
    // will be caught by clap.rs
    if args.occurrences_of("pages_access_level") > 0 {
        p.pages_access_level(
            feature_access_level_public_from_str(args.value_of("pages_access_level").unwrap())
                .unwrap(),
        );
    }

    // specific conversion to merge_method enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("merge_method") > 0 {
        p.merge_method(merge_method_from_str(args.value_of("merge_method").unwrap()).unwrap());
    }

    // specific conversion to build_git_strategy enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("pipeline_git_strategy") > 0 {
        p.build_git_strategy(
            pipeline_git_strategy_from_str(args.value_of("pipeline_git_strategy").unwrap())
                .unwrap(),
        );
    }

    if args.occurrences_of("tags") > 0 {
        for t in values_t_or_exit!(args, "tag", String) {
            p.tag(t);
        }
    }

    // arg "name" is enforced by clap.rs so unwrap() is safe...
    let endpoint = p.name(args.value_of("name").unwrap()).build().unwrap();

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    // TODO: Consider changing return value to Result<serde_json::Value> to get raw json.
    // TODO: fix unwrap() to check errors
    let project: Project = endpoint.query(&gitlabclient)
        .context("Failed to create project - check for name or path clashes on the server")?;

    println!("Project id: {}", project.id);
    println!("Project URL: {}", project.web_url);
    Ok(())
}

#[cfg(test)]
mod project_create_unit_tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    use anyhow::anyhow;
    use clap::SubCommand as ClapSubCommand;
    use gitlab::types::*;
    use serde::de::DeserializeOwned;

    use crate::gitlab::Project;
    use crate::subcommand::SubCommand;
    use crate::cmds::project;

    use super::*;

    struct GitlabWithMockProject {
        project: Result<Project>,
    }

    impl IfGitLabCreateProject for GitlabWithMockProject {
        fn create_project<N: AsRef<str>, P: AsRef<str>>(
            &self,
            _name: N,
            _path: Option<P>,
            _params: Option<CreateProjectParams>,
        ) -> Result<Project> {
            match &self.project {
                Ok(p) => Ok(p.clone()),
                Err(e) => Err(anyhow!("{}", e)),
            }
        }
    }

    fn load_mock_from_disk<P: AsRef<Path>, T>(path: P) -> T
    where
        T: DeserializeOwned,
    {
        // Open the file in read-only mode with buffer.
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `Project`.
        serde_json::from_reader(reader).unwrap()
    }

    #[test]
    fn test_create_basic_project() {
        // GIVEN:
        let p_cmd = project::Project{
            clap_cmd: ClapSubCommand::with_name("project")
        };
        let args = p_cmd
            .gen_clap_command()
            .get_matches_from(vec!["project", "create", "project_name"]);
        let matches = args.subcommand_matches("create");

        let mock_project: Project = load_mock_from_disk("tests/data/project.json");

        let g = GitlabWithMockProject {
            project: Ok(mock_project),
        };

        // WHEN:
        let p = create_project_cmd(matches.unwrap().clone(), g);

        // THEN:
        assert!(p.is_ok())
    }
}
