use anyhow::{Context, Result};
use clap::{value_t_or_exit, values_t_or_exit};

use crate::gitlab::{
    feature_visibility_level_from_str,
    merge_method_from_str,
    visibilily_level_from_str,
    pipeline_git_strategy_from_str,

    CreateProjectParams,
    IfGitLabCreateProject};

pub fn create_project_cmd(args: clap::ArgMatches, gitlab: impl IfGitLabCreateProject) -> Result<()> {
    let mut params = &mut CreateProjectParams::builder();

    for arg in &args.args {
        let (key, _) = arg;
        match *key {
            // basic casting to URL
            "import_url" => params = params.import_url(value_t_or_exit!(args, "import_url", url::Url)),

            // basic casting to u64
            "namespace_id" => params = params.namespace_id(value_t_or_exit!(args, "namespace_id", u64)),
            "build_timeout" => params = params.build_timeout(value_t_or_exit!(args, "build_timeout", u64)),

            // basic casting to bool for flags passed
            // TODO: check occurences_of() instead
            "enable_lfs" => params = params.lfs_enabled(value_t_or_exit!(args, "enable_lfs", bool)),
            "enable_request_access" => params = params.request_access_enabled(value_t_or_exit!(args, "enable_request_access", bool)),
            "enable_container_registry" => params = params.container_registry_enabled(value_t_or_exit!(args, "enable_container_registry", bool)),
            "print_merge_request_url" => params = params.printing_merge_request_link_enabled(value_t_or_exit!(args, "print_merge_request_url", bool)),
            "enable_auto_devops" => params = params.auto_devops_enabled(value_t_or_exit!(args, "enable_auto_devops", bool)),
            "enable_shared_runners" => params = params.shared_runners_enabled(value_t_or_exit!(args, "enable_shared_runners", bool)),
            "enable_public_builds" => params = params.public_builds(value_t_or_exit!(args, "public_builds", bool)),
            "resolve_old_discussions" => params = params.resolve_outdated_diff_discussions(value_t_or_exit!(args, "resolve_old_discussions", bool)),
            "only_merge_on_good_ci" => params = params.only_allow_merge_if_pipeline_succeeds(value_t_or_exit!(args, "only_merge_on_good_ci", bool)),
            "only_merge_on_closed_discussions" => params = params.only_allow_merge_if_all_discussions_are_resolved(value_t_or_exit!(args, "only_merge_on_closed_discussions", bool)),
            "auto_close_referenced_issues" => params = params.autoclose_referenced_issues(value_t_or_exit!(args, "auto_close_referenced_issues", bool)),
            "auto_cancel_pending_pipelines" => params = params.auto_cancel_pending_pipelines(value_t_or_exit!(args, "auto_cancel_pending_pipelines", bool)),

            // no casting, straight string
            "description" => params = params.description(args.value_of("description").unwrap()),
            "default_branch" => params = params.default_branch(args.value_of("default_branch").unwrap()),
            "build_coverage_regex" => params = params.build_coverage_regex(args.value_of("build_coverage_regex").unwrap()),
            "ci_config_path" => params = params.ci_config_path(args.value_of("ci_config_path").unwrap()),
            "auto_devops_deploy_strategy" => params = params.auto_devops_deploy_strategy(args.value_of("auto_devops_deploy_strategy").unwrap()),

            // specific cast to visibilily_level enum - unwrap()'s are safe as problems will be caught by clap.rs
            "visibility" => params = params.visibility(visibilily_level_from_str(args.value_of("visibility").unwrap()).unwrap()),

            // specific cast to feature_visibilily_level enum - unwrap()'s are safe as problems will be caught by clap.rs
            "repo_access_level" => params = params.repository_access_level(feature_visibility_level_from_str(args.value_of("repo_access_level").unwrap()).unwrap()),
            "mr_access_level" => params = params.merge_requests_access_level(feature_visibility_level_from_str(args.value_of("mr_access_level").unwrap()).unwrap()),
            "builds_access_level" => params = params.builds_access_level(feature_visibility_level_from_str(args.value_of("builds_access_level").unwrap()).unwrap()),
            "wiki_access_level" => params = params.wiki_access_level(feature_visibility_level_from_str(args.value_of("wiki_access_level").unwrap()).unwrap()),
            "snippets_access_level" => params = params.snippets_access_level(feature_visibility_level_from_str(args.value_of("snippets_access_level").unwrap()).unwrap()),
            "pages_access_level" => params = params.pages_access_level(feature_visibility_level_from_str(args.value_of("pages_access_level").unwrap()).unwrap()),

            // specific cast to merge_method enum - unwrap()'s are safe as problems will be caught by clap.rs
            "merge_method" => params = params.merge_method(merge_method_from_str(args.value_of("merge_method").unwrap()).unwrap()),

            // specific cast to build_git_strategy enum - unwrap()'s are safe as problems will be caught by clap.rs
            "pipeline_git_strategy" => params = params.build_git_strategy(pipeline_git_strategy_from_str(args.value_of("pipeline_git_strategy").unwrap()).unwrap()),

            // cast to a Vec of Strings
            "tag_list" => params = params.tag_list(values_t_or_exit!(args, "tag_list", String)),
            "container_expiration_policy" => params = params.container_expiration_policy_attributes(values_t_or_exit!(args, "container_expiration_policy", String)),

            _ => unreachable!(),
        }
    }

    // TODO: Consider changing return value to Result<serde_json::Value> to get raw json.
    let project = gitlab
        .create_project(args.value_of("name").unwrap(), args.value_of("path"), Some(params.build().unwrap()))
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
    use clap::App;
    use gitlab::types::*;
    use serde::de::DeserializeOwned;

    use crate::gitlab::Project;

    use super::*;

    struct GitlabWithMockProject {
        project: Result<Project>,
    }

    impl IfGitLabCreateProject for GitlabWithMockProject {
        fn create_project<N: AsRef<str>, P: AsRef<str>>(&self, name: N, path: Option<P>, params: Option<CreateProjectParams>) -> Result<Project> {
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
        let a = App::new("create")
            .arg(clap::Arg::with_name("name").help("Project name").takes_value(true).required(true))
            .get_matches_from(vec!["create", "proj_name"]);
        println!("args = {:?}", a);

        let mock_project: Project = load_mock_from_disk("tests/data/project.json");

        let g = GitlabWithMockProject { project: Ok(mock_project) };

        // WHEN:
        let p = create_project_cmd(a, g);

        // THEN:
    }
}
