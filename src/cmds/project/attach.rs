//! This module implements attaching (ie associating) a project in GitLab to a local git repo.
//!
//! It does this by referring to the local git remote (which must be set) and looking it up on the
//! GitLab server. If found, it will update and persist local repo-specific config to contain the
//! GitLab project's ID so that other project-specific commands can use it.
use anyhow::{anyhow, Context, Result};
use clap::value_t;
use git2::Repository;
use graphql_client::GraphQLQuery;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use crate::config;
use crate::gitlab::Labels as GLLabels;
use crate::gitlab::ProjectMembers as GLMembers;
use crate::gitlab::Project as GLProject;
use crate::gitlab::{api, Query};
use crate::gitlab;
use crate::utils;

/// This maps to GitLab's two Project fields: `sshUrlToRepo` and `httpUrlToRepo`
///
/// Note that from a git perspective, server protocols under the `ssh` type in gitlab could either
/// be prefixed `git@` or `ssh://`. We lump both of these types in as _SSH_ as GitLab does although
/// technically, they are different protocols as discussed
/// [here](https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols).
///
/// Note also that HTTPS is (obviously) covered under the _HTTP_ protocol.
#[derive(Debug)]
#[derive(PartialEq)]
enum RemoteType {
    HTTP,
    SSH,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/projects_with_remotes.graphql",
    response_derives = "Debug"
)]
struct ProjectsWithRemotes;

/// Return the push url for the `origin` remote, if set.
fn get_git_remote(config: &config::Config) -> Option<String> {
    let repo = Repository::open(config.repo_path.as_ref()?).ok()?;
    let origin = repo.find_remote("origin").ok()?;
    let remote_str  = String::from(origin.url()?);
    Some(remote_str)
}

/// Given a remote url, figure out what type it is, and what search term to find it with
fn get_search_param_and_remote_type(url: &str) -> (RemoteType, String) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(?P<remote>ssh|git|http)[^\s]+/(?P<search>[^\s]+)\.git$").unwrap();
    }

    let caps = RE.captures(url).unwrap();

    let r_type = match caps.name("remote").unwrap().as_str() {
        "http" => RemoteType::HTTP,
        "ssh" => RemoteType::SSH,
        "git" => RemoteType::SSH,
        _ => unreachable!(),
    };

    (r_type, caps.name("search").unwrap().as_str().to_string())
}

/// Get list of remote-pairs from server which match search string
fn get_remotes_from_server(search_str: &str, gitlabclient: &gitlab::Client) -> Result<projects_with_remotes::ResponseData> {
    let query_body = ProjectsWithRemotes::build_query(
        projects_with_remotes::Variables {
            search_str: search_str.to_string(),
        }
    );

    debug!("graphql search string: {}", search_str);

    let response = gitlabclient.graphql::<ProjectsWithRemotes>(&query_body)
        .context("GraphQL error when looking for Projects with matching remotes")?;

    Ok(response)
}

/// Parse project id from `gid://gitlab/Project/<pid>` string
fn parse_pid_from_gid(gid_str: &str) -> u64 {
    let v: Vec<&str> = gid_str.rsplit('/').collect();
    v[0].parse::<u64>().unwrap()
}

/// Return the GitLab project ID
fn find_project_id(r_type: RemoteType, url: &str, remotes: projects_with_remotes::ResponseData) -> Result<u64> {

    let search_array = remotes.projects.unwrap().nodes.unwrap();

    for item in search_array {

        let item = item.unwrap();

        let gl_remote = match r_type {
            RemoteType::HTTP => item.http_url_to_repo.unwrap(),
            RemoteType::SSH => item.ssh_url_to_repo.unwrap(),
        };

        if gl_remote == url {
            return Ok(parse_pid_from_gid(&item.id))
        }
    }

    Err(anyhow!("Counldn't find matching project ID"))
}

/// Look up the project ID on the GitLab server from a git remote url.
fn get_proj_id_by_remote(url: &str, gitlabclient: &gitlab::Client) -> Result<u64> {
    trace!("url: {:#?}", url);

    let (r_type, search_str) = get_search_param_and_remote_type(url);

    let response = get_remotes_from_server(&search_str, gitlabclient)?;
    trace!("response: {:#?}", response);

    let p_id = find_project_id(r_type, url, response)?;

    Ok(p_id)
}

// Note that this function implements a workaround for a buggy Gitlab API. The include ancestors
// endpoint should _include_ ancestors, but instead it returns _only_ ancestors(!) so doing two
// calls and merging the results.
fn get_project_members(project_id: u64, max_members: u64, gitlabclient: &gitlab::Client) -> Result<Vec<String>> {

    #[derive(Deserialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct Member {
        id: u64,
        username: String
    }

    // ancestor members
    let mut a_members_builder = GLMembers::all_builder();
    let a_endpoint = a_members_builder.project(project_id).build()
        .map_err(|e| anyhow!("Could not fetch project members from server.\n {}",e))?;

    debug!("ancestor members endpoint: {:#?}", a_endpoint);

    let mut a_members: Vec<Member> = api::paged(a_endpoint, api::Pagination::Limit(max_members as usize))
        .query(gitlabclient)
        .context("Failed to query project members")?;

    debug!("ancestor members: {:#?}", a_members);

    // project members
    let mut p_members_builder = GLMembers::builder();
    let p_endpoint = p_members_builder.project(project_id).build()
        .map_err(|e| anyhow!("Could not fetch project members from server.\n {}",e))?;

    debug!("project members endpoint: {:#?}", p_endpoint);

    let mut p_members: Vec<Member> = api::paged(p_endpoint, api::Pagination::Limit(max_members as usize))
        .query(gitlabclient)
        .context("Failed to query project members")?;

    debug!("project members: {:#?}", p_members);

    a_members.append(&mut p_members);
    a_members.sort_by(|a,b| a.username.cmp(&b.username));
    a_members.dedup();

    debug!("final sorted and deduped members: {:#?}", a_members);

    Ok(a_members.iter().map(|m| format!("{}:{}", m.id.to_string(), m.username.clone())).collect())
}

fn get_project_path_with_namespace(project_id: u64, gitlabclient: &gitlab::Client) -> Result<String> {
    let mut project_builder  = GLProject::builder();
    let endpoint = project_builder.project(project_id).build()
        .map_err(|e| anyhow!("Could not fetch project from server.\n {}",e))?;

    debug!("endpoint: {:#?}", endpoint);

    #[derive(Deserialize, Debug)]
    struct Project {
        path_with_namespace: String
    }

    let project: Project = endpoint
        .query(gitlabclient)
        .context("Failed to query project")?;

    debug!("project: {:#?}", project);
    Ok(project.path_with_namespace)
}

fn get_project_defaultbranch(project_id: u64, gitlabclient: &gitlab::Client) -> Result<String> {
    let mut project_builder  = GLProject::builder();
    let endpoint = project_builder.project(project_id).build()
        .map_err(|e| anyhow!("Could not fetch project from server.\n {}",e))?;

    debug!("endpoint: {:#?}", endpoint);

    #[derive(Deserialize, Debug)]
    struct Project {
        default_branch: String
    }

    let project: Project = endpoint
        .query(gitlabclient)
        .context("Failed to query project")?;

    debug!("project: {:#?}", project);
    Ok(project.default_branch)
}

fn get_project_labels(project_id: u64, max_labels: u64, gitlabclient: &gitlab::Client) -> Result<Vec<String>> {
    let mut labels_builder = GLLabels::builder();
    let endpoint = labels_builder.project(project_id).build()
        .map_err(|e| anyhow!("Could not fetch project labels from server.\n {}",e))?;

    debug!("endpoint: {:#?}", endpoint);

    #[derive(Deserialize, Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct Label {
        name: String
    }

    let mut labels: Vec<Label> = api::paged(endpoint, api::Pagination::Limit(max_labels as usize))
        .query(gitlabclient)
        .context("Failed to query project labels")?;

    labels.sort();
    debug!("labels: {:#?}", labels);
    Ok(labels.iter().map(|l| l.name.clone()).collect())
}

pub fn attach_project_cmd(args: clap::ArgMatches, mut config: config::Config, gitlabclient: gitlab::Client) -> Result<()> {
    // if not inside local repo error and exit
    config.repo_path.as_ref().ok_or_else(|| anyhow!("Local repo not found. Are you in the correct directory?"))?;

    debug!("config: {:#?}", &config);

    let project_id = match (&get_git_remote(&config), &args) {
        (Some(r), a) if !a.is_present("project_id") => {
            get_proj_id_by_remote(r, &gitlabclient)
                .with_context(|| format!("Could not look up GitLab project using 'origin' remote '{}'", r))
                .context("Your GitLab server is probably not at a version with decent GraphQL support.")
        },
        (_, a) if a.is_present("project_id") => {
            a.value_of("project_id").unwrap().parse::<u64>().map_err(|e| anyhow!(e))
        },
        (r, a) => {
            trace!("remote_url: {:#?}", r);
            trace!("args: {:#?}", a);
            Err(anyhow!("Git remote 'origin' not found. Set the remote or pass the project details explicitly"))
        }
    }?;

    config.projectid = Some(project_id);
    config.defaultbranch = get_project_defaultbranch(project_id, &gitlabclient).ok();
    config.path_with_namespace = get_project_path_with_namespace(project_id, &gitlabclient).ok();
    config.labels = get_project_labels(project_id, value_t!(args, "max_labels", u64).unwrap(), &gitlabclient)?;
    config.members = get_project_members(project_id, value_t!(args, "max_members", u64).unwrap(), &gitlabclient)?;
    config.save(config::GitConfigSaveableLevel::Repo)?;

    let out_vars = vec!(("project_id".to_string(), project_id.to_string())).into_iter();
    utils::write_short_output(config.format, out_vars)
}

#[cfg(test)]
mod project_attach_unit_tests {
    use rstest::*;

    use super::*;

    #[rstest(
    url, r_type, search_str,
    case("git@gitlab.com:aiganym_sag/hostel-management-system-master.git", RemoteType::SSH, "hostel-management-system-master"),
    case("git@gitlab.com:aiganym_sag/hostel/management/system-master.git", RemoteType::SSH, "system-master"),
    case("ssh://git@gitlab.com:one/two/three.git", RemoteType::SSH, "three"),
    case("ssh://git@gitlab.com:2222/one/two/three.git", RemoteType::SSH, "three"),
    case("https://gitlab.com/jandamuda0400/berat-badan-dan-jerawat.git", RemoteType::HTTP, "berat-badan-dan-jerawat"),
    case("https://gitlab.com/jandamuda0400/berat/badan/dan/jerawat.git", RemoteType::HTTP, "jerawat"),
    case("http://gitlab.com/jandamuda0400/berat-badan-dan-jerawat.git", RemoteType::HTTP, "berat-badan-dan-jerawat"),

    )]
    fn test_get_search_param_and_remote_type(url: &str, r_type: RemoteType, search_str: &str) {
        let (t,s) = get_search_param_and_remote_type(url);
        assert_eq!(t, r_type);
        assert_eq!(s, search_str);
    }

    #[rstest(
    gid_str, pid,
    case("gid://gitlab/Project/12345", 12345),
    )]
    fn test_parse_pid_from_gid(gid_str: &str, pid: u64) {
        assert_eq!(pid, parse_pid_from_gid(gid_str));
    }

    #[rstest(
    pid, r_type, url, remotes,
    case(23456, RemoteType::HTTP, "https://gitlab.com:one/two/four.git",
        projects_with_remotes::ResponseData {
            projects:
                Some(
                    projects_with_remotes::ProjectsWithRemotesProjects {
                        nodes:
                            Some(
                                vec!(
                                    Some(
                                        projects_with_remotes::ProjectsWithRemotesProjectsNodes {
                                            id: "gid://gitlab/Project/12345".to_string(),
                                            ssh_url_to_repo: Some("ssh://git@gitlab.com:one/two/three.git".to_string()),
                                            http_url_to_repo: Some("https://gitlab.com:one/two/three.git".to_string()),
                                        }
                                    ),
                                    Some(
                                        projects_with_remotes::ProjectsWithRemotesProjectsNodes {
                                            id: "gid://gitlab/Project/23456".to_string(),
                                            ssh_url_to_repo: Some("ssh://git@gitlab.com:one/two/four.git".to_string()),
                                            http_url_to_repo: Some("https://gitlab.com:one/two/four.git".to_string()),
                                        }
                                    ),
                                )
                            )
                    }
                )
        }
    ),
    case(12345, RemoteType::SSH, "ssh://git@gitlab.com:one/two/three.git",
        projects_with_remotes::ResponseData {
            projects:
                Some(
                    projects_with_remotes::ProjectsWithRemotesProjects {
                        nodes:
                            Some(
                                vec!(
                                    Some(
                                        projects_with_remotes::ProjectsWithRemotesProjectsNodes {
                                            id: "gid://gitlab/Project/12345".to_string(),
                                            ssh_url_to_repo: Some("ssh://git@gitlab.com:one/two/three.git".to_string()),
                                            http_url_to_repo: Some("https://gitlab.com:one/two/three.git".to_string()),
                                        }
                                    ),
                                    Some(
                                        projects_with_remotes::ProjectsWithRemotesProjectsNodes {
                                            id: "gid://gitlab/Project/23456".to_string(),
                                            ssh_url_to_repo: Some("ssh://git@gitlab.com:one/two/four.git".to_string()),
                                            http_url_to_repo: Some("https://gitlab.com:one/two/four.git".to_string()),
                                        }
                                    ),
                                )
                            )
                    }
                )
        }
    ),
    )]
    fn test_find_project_id_good(pid: u64, r_type: RemoteType, url: &str, remotes: projects_with_remotes::ResponseData) {
        assert_eq!(pid, find_project_id(r_type, url, remotes).unwrap());
    }
    #[rstest(
    r_type, url, remotes,
    case(RemoteType::SSH, "ssh://git@gitlab.com:one/two/three.git",
        projects_with_remotes::ResponseData {
            projects:
                Some(
                    projects_with_remotes::ProjectsWithRemotesProjects {
                        nodes:
                            Some(
                                vec!()
                            )
                    }
                )
        }
    ),
    )]
    fn test_find_project_id_bad(r_type: RemoteType, url: &str, remotes: projects_with_remotes::ResponseData) {
        assert!(
            find_project_id(r_type, url, remotes).is_err(),
        );
    }
}
