use anyhow::{Context, Result};
use clap::{value_t_or_exit, values_t_or_exit};

use crate::gitlab::{
    feature_visibility_level_from_str, merge_method_from_str, pipeline_git_strategy_from_str,
    visibility_level_from_str, CreateProjectParams, IfGitLabCreateProject,
};

pub fn create_project_cmd(
    args: clap::ArgMatches,
    gitlab: impl IfGitLabCreateProject,
) -> Result<()> {
    let mut p = CreateProjectParams::builder();

    // basic casting to URL
    if args.occurrences_of("import_url") > 0 {
        debug!("import_url");
        p.import_url(value_t_or_exit!(args, "import_url", url::Url));
    }

    // basic casting to u64
    if args.occurrences_of("namespace_id") > 0 {
        debug!("namespace_id");
        p.namespace_id(value_t_or_exit!(args, "namespace_id", u64));
    }
    if args.occurrences_of("build_timeout") > 0 {
        debug!("build_timeout");
        p.build_timeout(value_t_or_exit!(args, "build_timeout", u64));
    }

    // basic boolean flags
    if args.occurrences_of("enable_lfs") > 0 {
        debug!("enable_lfs");
        p.lfs_enabled(true);
    }
    if args.occurrences_of("enable_request_access") > 0 {
        debug!("enable_request_access");
        p.request_access_enabled(true);
    }
    if args.occurrences_of("enable_container_registry") > 0 {
        debug!("enable_container_registry");
        p.container_registry_enabled(true);
    }
    if args.occurrences_of("print_merge_request_url") > 0 {
        debug!("print_merge_request_url");
        p.printing_merge_request_link_enabled(true);
    }
    if args.occurrences_of("enable_auto_devops") > 0 {
        debug!("enable_auto_devops");
        p.auto_devops_enabled(true);
    }
    if args.occurrences_of("enable_shared_runners") > 0 {
        debug!("enable_shared_runners");
        p.shared_runners_enabled(true);
    }
    if args.occurrences_of("enable_public_builds") > 0 {
        debug!("enable_public_builds");
        p.public_builds(true);
    }
    if args.occurrences_of("resolve_old_discussions") > 0 {
        debug!("resolve_old_discussions");
        p.resolve_outdated_diff_discussions(true);
    }
    if args.occurrences_of("only_merge_on_good_ci") > 0 {
        debug!("only_merge_on_good_ci");
        p.only_allow_merge_if_pipeline_succeeds(true);
    }
    if args.occurrences_of("only_merge_on_closed_discussions") > 0 {
        debug!("only_merge_on_closed_discussions");
        p.only_allow_merge_if_all_discussions_are_resolved(true);
    }
    if args.occurrences_of("auto_close_referenced_issues") > 0 {
        debug!("auto_close_referenced_issues");
        p.autoclose_referenced_issues(true);
    }
    if args.occurrences_of("auto_cancel_pending_pipelines") > 0 {
        debug!("auto_cancel_pending_pipelines");
        p.auto_cancel_pending_pipelines(true);
    }

    // no casting, straight string
    if args.occurrences_of("description") > 0 {
        debug!("description");
        p.description(args.value_of("description").unwrap());
    }
    if args.occurrences_of("default_branch") > 0 {
        debug!("default_branch");
        p.default_branch(args.value_of("default_branch").unwrap());
    }
    if args.occurrences_of("build_coverage_regex") > 0 {
        debug!("build_coverage_regex");
        p.build_coverage_regex(args.value_of("build_coverage_regex").unwrap());
    }
    if args.occurrences_of("ci_config_path") > 0 {
        debug!("ci_config_path");
        p.ci_config_path(args.value_of("ci_config_path").unwrap());
    }
    if args.occurrences_of("auto_devops_deploy_strategy") > 0 {
        debug!("auto_devops_deploy_strategy");
        p.auto_devops_deploy_strategy(args.value_of("auto_devops_deploy_strategy").unwrap());
    }

    // specific conversion to visibilily_level enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("visibility") > 0 {
        debug!("visibility");
        p.visibility(visibility_level_from_str(args.value_of("visibility").unwrap()).unwrap());
    }

    // FIXME these don't work for some reason -- try doing another post after the create to set
    // visibility requirements
    // specific conversion to feature_visibilily_level enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("repo_access_level") > 0 {
        debug!("repo_access_level");
        p.repository_access_level(
            feature_visibility_level_from_str(args.value_of("repo_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("issues_access_level") > 0 {
        debug!("issues_access_level");
        p.issues_access_level(
            feature_visibility_level_from_str(args.value_of("issues_access_level").unwrap())
                .unwrap(),
        );
    }
    if args.occurrences_of("mr_access_level") > 0 {
        debug!("mr_access_level");
        p.merge_requests_access_level(
            feature_visibility_level_from_str(args.value_of("mr_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("builds_access_level") > 0 {
        debug!("builds_access_level");
        p.builds_access_level(
            feature_visibility_level_from_str(args.value_of("builds_access_level").unwrap())
                .unwrap(),
        );
    }
    if args.occurrences_of("wiki_access_level") > 0 {
        debug!("wiki_access_level");
        p.wiki_access_level(
            feature_visibility_level_from_str(args.value_of("wiki_access_level").unwrap()).unwrap(),
        );
    }
    if args.occurrences_of("snippets_access_level") > 0 {
        debug!("snippets_access_level");
        p.snippets_access_level(
            feature_visibility_level_from_str(args.value_of("snippets_access_level").unwrap())
                .unwrap(),
        );
    }
    if args.occurrences_of("pages_access_level") > 0 {
        debug!("pages_access_level");
        p.pages_access_level(
            feature_visibility_level_from_str(args.value_of("pages_access_level").unwrap())
                .unwrap(),
        );
    }

    // specific conversion to merge_method enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("merge_method") > 0 {
        debug!("merge_method");
        p.merge_method(merge_method_from_str(args.value_of("merge_method").unwrap()).unwrap());
    }

    // specific conversion to build_git_strategy enum - unwrap()'s are safe as problems will be caught by clap.rs
    if args.occurrences_of("pipeline_git_strategy") > 0 {
        debug!("pipeline_git_strategy");
        p.build_git_strategy(
            pipeline_git_strategy_from_str(args.value_of("pipeline_git_strategy").unwrap())
                .unwrap(),
        );
    }

    // FIXME array passing doesn't seem to work here... Investigate!
    // Upstream BUG: https://gitlab.kitware.com/utils/rust-gitlab/-/issues/34
    if args.occurrences_of("tag_list") > 0 {
        debug!("tag_list");
        p.tag_list(values_t_or_exit!(args, "tag_list", String));
    }
    if args.occurrences_of("container_expiration_policy") > 0 {
        debug!("container_expiration_policy");
        p.container_expiration_policy_attributes(values_t_or_exit!(
            args,
            "container_expiration_policy",
            String
        ));
    }

    let params = p.build();
    debug!("params: {:#?}", params);
    debug!("args: {:#?}", args);
    // TODO: Consider changing return value to Result<serde_json::Value> to get raw json.
    let project = gitlab
        .create_project(
            args.value_of("name").unwrap(),
            args.value_of("path").or_else(|| None),
            Some(params.unwrap()),
        )
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
