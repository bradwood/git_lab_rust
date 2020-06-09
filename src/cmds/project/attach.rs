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

#[derive(Debug)]
#[derive(PartialEq)]
enum RemoteType {
    HTTPS,
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
        static ref RE: Regex = Regex::new(r"^(?P<remote>git|https)[^\s]+/(?P<search>[^\s]+)\.git$").unwrap();
    }

    let caps = RE.captures(url).unwrap();

    let r_type = match caps.name("remote").unwrap().as_str() {
        "https" => RemoteType::HTTPS,
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

/// Parse project id from `gid://gitlab/Project/<id>` string
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
            RemoteType::HTTPS => item.http_url_to_repo.unwrap(),
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
