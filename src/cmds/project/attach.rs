//! This module implements attaching (ie associating) a project in GitLab to a local git repo.
//!
//! It does this by referring to the local git remote (which must be set) and looking it up on the
//! GitLab server. If found, it will update and persist local repo-specific config to contain the
//! GitLab project's ID so that other project-specific commands can use it.
use anyhow::{anyhow, Context, Result};
use git2::Repository;
use graphql_client::GraphQLQuery;
use lazy_static::lazy_static;
use regex::Regex;

use crate::config;
use crate::gitlab;

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
    query_path = "src/graphql/ProjectsWithRemotes.graphql",
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
fn get_remotes_from_server(search_str: &str, gitlabclient: gitlab::Client) -> Result<projects_with_remotes::ResponseData> {
    let query_body = ProjectsWithRemotes::build_query(
        projects_with_remotes::Variables {
            search_str: search_str.to_string(),
        }
    );

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
fn get_proj_id_by_remote(url: &str, gitlabclient: gitlab::Client) -> Result<u64> {
    trace!("url: {:#?}", url);

    let (r_type, search_str) = get_search_param_and_remote_type(url);

    let response = get_remotes_from_server(&search_str, gitlabclient)?;
    trace!("response: {:#?}", response);

    let p_id = find_project_id(r_type, url, response)?;

    Ok(p_id)

}

pub fn attach_project_cmd(args: clap::ArgMatches, mut config: config::Config, gitlabclient: gitlab::Client) -> Result<()> {
    // if not inside local repo error and exit
    config.repo_path.as_ref().ok_or_else(|| anyhow!("Local repo not found. Are you in the correct directory?"))?;

    let project_id = match (&get_git_remote(&config), args) {
        (Some(r), a) if !a.is_present("name") && !a.is_present("id") => {
            get_proj_id_by_remote(r, gitlabclient)
                .with_context(|| format!("Could not look up GitLab project using 'origin' remote '{}'", r))
        },
        (_, a) if a.is_present("id") => {
            a.value_of("id").unwrap().parse::<u64>().map_err(|e| anyhow!(e))
        },
        (r, a) => {
            trace!("remote_url: {:#?}", r);
            trace!("args: {:#?}", a);
            Err(anyhow!("Git remote 'origin' not found. Set the remote or pass the project details explicitly"))
        }
    }?;
    config.projectid = Some(project_id);
    config.save(config::GitConfigSaveableLevel::Repo)?;
    println!("Attached to GitLab Project ID: {}", project_id);
    Ok(())
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
