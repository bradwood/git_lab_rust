use anyhow::{Context, Result};

use crate::gitlab::{CreateProjectParams, IfGitLabCreateProject};

pub fn create_project_cmd(
    args: clap::ArgMatches,
    gitlab: impl IfGitLabCreateProject,
) -> Result<()> {
    let mut params = &mut CreateProjectParams::builder();

    if args.is_present("description") {
        params = params.description(args.value_of("description").unwrap());
        trace!("description: {}", args.value_of("description").unwrap());
    }

    // TODO: add various arg parsing code

    // TODO: Consider changing return value to Result<serde_json::Value> to get raw json.
    let project = gitlab
        .create_project(
            args.value_of("name").unwrap(),
            args.value_of("path"),
            Some(params.build().unwrap()),
        )
        .context("Failed to create project - check for name or path clashes on the server")?;

    println!("Project id: {}", project.id);
    println!("Project URL: {}", project.web_url);
    Ok(())
}

/*
#[cfg(test)]
mod project_create_unit_tests {
    use crate::gitlab::Project;
    use clap::App;
    use lazy_static::*;
    use anyhow::anyhow;

    use chrono::TimeZone;
    use chrono::Utc;
    use gitlab::types::*;

    use super::*;

    lazy_static! {
        static ref DEFAULT_MOCK_PROJECT: Project =
        Project {
            id: ProjectId::new(1u64),
            description: Some("description".to_string()),
            default_branch: Some("master".to_string()),
            tag_list: vec!["tag".to_string(),"list".to_string()],
            archived: false,
            empty_repo: true,
            visibility: VisibilityLevel::Public,
            ssh_url_to_repo: "ssh://url.to.repo/path.git".to_string(),
            http_url_to_repo: "https://url.to.repo/path.git".to_string(),
            web_url: "https://url.to.repo/path".to_string(),
            readme_url: Some("https://url.to.repo/path/README.md".to_string()),
            owner: Some(UserBasic {
                username: "username".to_string(),
                name: "name".to_string(),
                id: UserId::new(10u64),
                state: UserState::Active,
                avatar_url: Some("http://avatar.url/user".to_string()),
                web_url: "https://url.to/user".to_string(),
            }),
            name: "name".to_string(),
            name_with_namespace: "name_with_namespace".to_string(),
            path: "path".to_string(),
            path_with_namespace: "path_with_namespace".to_string(),
            container_registry_enabled: Some(true),
            created_at: Utc.ymd(2001, 9, 9).and_hms_milli(1, 46, 40, 555),
            last_activity_at: Utc.ymd(2009, 9, 9).and_hms_milli(1, 46, 40, 555),
            shared_runners_enabled: true,
            lfs_enabled: true,
            creator_id: UserId::new(23u64),
            namespace: Namespace {
                id: 324u64,
                full_path: "namespace/full/path".to_string(),
                path: "namespace_path".to_string(),
                name: "namespace_name".to_string(),
                avatar_url: Some("http://namespace.avatar.url/user".to_string()),
                web_url: "https://namespace.url.to/user".to_string(),
                kind: NamespaceKind::User,
                members_count_with_descendants: None // (only available to admins
            },
            forked_from_project: None, // can be Some(BasicProjectDetails)
            avatar_url: Some("http://project.avatar.url/user".to_string()),
            ci_config_path: Some(".gitlab-ci.yml".to_string()),
            import_error: None, // can be Some(String) with error msg in it
            star_count: 32u64,
            forks_count: 3u64,
            open_issues_count: Some(33u64),
            runners_token: Some("someuuidliketoken".to_string()),
            public_jobs: true,
            shared_with_groups:vec![
                SharedGroup {
                    group_id: GroupId::new(332u64),
                    group_name: "shared_group".to_string(),
                    group_access_level: 23u64,
                }
            ],
            only_allow_merge_if_pipeline_succeeds: Some(true),
            only_allow_merge_if_all_discussions_are_resolved: Some(true),
            remove_source_branch_after_merge: Some(true),
            printing_merge_request_link_enabled: Some(true),
            request_access_enabled: true,
            resolve_outdated_diff_discussions: Some(true),
            jobs_enabled: true,
            issues_enabled: true,
            merge_requests_enabled: true,
            snippets_enabled: true,
            wiki_enabled: true,
            builds_access_level: FeatureVisibilityLevel::Public,
            issues_access_level: FeatureVisibilityLevel::Public,
            merge_requests_access_level: FeatureVisibilityLevel::Public,
            repository_access_level: FeatureVisibilityLevel::Public,
            snippets_access_level: FeatureVisibilityLevel::Public,
            wiki_access_level: FeatureVisibilityLevel::Public,
            merge_method: Some("merge".to_string()),
            statistics: None, // could be Some(ProjectStatistics)
            permissions: Some(
                Permissions {
                    project_access: Some(MemberAccess{ access_level: 3u64, notification_level: Some(1u64) }),
                    group_access: Some(MemberAccess{ access_level: 3u64, notification_level: Some(1u64) }),
                },
            ),
            _links: None, // not exposed by TP gitlab lib normally
        };
    }

    struct GitlabWithMockProject {
        project: Result<Project>,
    }

    impl IfGitLabCreateProject for GitlabWithMockProject {
        fn create_project<N: AsRef<str>, P: AsRef<str>>(
            &self,
            name: N,
            path: Option<P>,
            params: Option<CreateProjectParams>,
        ) -> Result<Project> {
            match &self.project {
                Ok(p) => Ok(p.clone()),
                Err(e) => Err(anyhow!("{}",e)),
            }
        }
    }

    #[test]
    fn test_create_basic_project() {
        // GIVEN:
        let a = App::new("project").get_matches_from(vec!["arg", "arg", "arg"]);

        let mocked_project = Project {
            id: ProjectId::new(1u64),
            description: Some("description".to_string()),
            default_branch: Some("master".to_string()),
            ..DEFAULT_MOCK_PROJECT.clone()
        };
        let g = GitlabWithMockProject {
            project: Ok(Project { ..mocked_project }),
        };

        // WHEN:
        let p = create_project_cmd(a, g);

        // THEN:
    }
}
*/
