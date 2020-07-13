use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context,  Result};
use clap::value_t;
use dialoguer::{Confirm, Input, Editor};
use git2::{Branch, Repository};
use graphql_client::GraphQLQuery;
use serde::Deserialize;
use slugify::slugify;

use crate::cmds::issue::generate_basic_issue_builder;
use crate::config;
use crate::gitlab::{Client, CreateMergeRequest, Query};
use crate::gitlab::Issue as GLIssue;
use crate::gitlab::Branch as GLBranch;
use crate::gitlab::CreateBranch as GLCreateBranch;
use crate::mr::MergeRequest;
use crate::utils;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/search_for_open_mr.graphql",
    response_derives = "Debug"
)]
struct SearchForOpenMr;

fn resolves_issue_mr_title(issue_title: Result<&str>) -> Result<String> {
    debug!("resolves_issue_mr_title");
    Ok(format!("Resolve \"{}\"", issue_title?))
}

/// Check if remote branch exists on the server
fn remote_branch_exists(p: u64, branch: &str, gitlabclient: &Client) -> bool {
    debug!("remote_branch_exists");
    #[derive(Deserialize, Debug)]
    struct Branch {}

    let mut b = GLBranch::builder();
    let endpoint = b.project(p).branch(branch).build().ok();

    if endpoint.is_none() { return false };

    let branch: Option<Branch> = endpoint
        .unwrap()
        .query(gitlabclient).ok();

    branch.is_some()
}

/// Check if there is an open merge request on the server
fn open_mr_on_branch(p: &str, branch: &str, gitlabclient: &Client) -> bool {
    debug!("open_mr_on_branch");
    let query_body = SearchForOpenMr::build_query(
        search_for_open_mr::Variables {
            search_str: Some(vec!(branch.to_string())),
            proj_path: p.to_string(),
        }
    );

    let mrs = gitlabclient.graphql::<SearchForOpenMr>(&query_body)
        .unwrap()
        .project.unwrap()
        .merge_requests.unwrap()
        .nodes.unwrap();

    if mrs.is_empty() { return false }

    mrs.into_iter()
        .map(|b| b.unwrap())
        .any(|b|
            b.state == search_for_open_mr::MergeRequestState::locked ||
            b.state == search_for_open_mr::MergeRequestState::opened)
}

fn get_commit_details(repo_path: &PathBuf) -> Result<(Option<String>, Option<String>)> {
    let repo = Repository::open(&repo_path)
        .context("Could not find local repo")?;
    let head = repo.head()
        .context("Could not find HEAD of local repo")?;
    let commit = head.peel_to_commit()
        .context("Could not exract commit from HEAD ref")?;
    let message = commit.message()
        .context("Could not exract commit message from message")?;
    let message_lines: Vec<&str> = message.lines().collect();

    if message_lines.len() >2 {
        Ok((
            Some(message_lines[0].to_string()),
            Some(message_lines[2..].join("\n"))
        ))

    } else {
        Ok((
            Some(message_lines[0].to_string()),
            None
        ))
    }
}

fn get_current_local_branch_name(repo_path: &PathBuf) -> Result<String> {
    debug!("get_current_local_branch_name");
    let repo = Repository::open(&repo_path)
        .context("Could not find local repo")?;
    let head = repo.head()
        .context("Could not find HEAD of local repo")?;

    if head.is_branch() {
        let b = Branch::wrap(head);
        let b_name = b.name()
            .context("Could not find the branch name of the current HEAD")?;
        let b_name = b_name.
            ok_or_else(|| anyhow!("Could not extract branch name"))?;
        Ok(b_name.to_string())
    } else {
        Err(anyhow!("Could not find current local branch"))
    }
}

fn get_current_remote_branch_name(repo_path: &PathBuf) -> Result<String> {
    debug!("get_current_remote_branch_name");

    let repo = Repository::open(&repo_path)
        .context("Could not find local repo")?;
    debug!("get_current_remote_branch_name - repo opened");

    let head = repo.head()
        .context("Could not find HEAD of local repo")?;
    debug!("get_current_remote_branch_name - found HEAD");

    if head.is_branch() {
        debug!("get_current_remote_branch_name - HEAD is branch");
        let b = Branch::wrap(head);
        debug!("get_current_remote_branch_name - got branch from HEAD");
        let upstream = b.upstream()
            .context("Could not find the upstream branch name of the current local branch")?;
        debug!("get_current_remote_branch_name - got upstream from branch pointing to head");
        let b_name = upstream.name()
            .context("Could not find the branch name of the remote branch")?;
        let name = b_name.
            ok_or_else(|| anyhow!("Could not extract branch name"))?;
        debug!("get_current_remote_branch_name - got upstream branch name: {}", name);
        if name.starts_with("origin/") {
            Ok(name.replacen("origin/","", 1))
        } else {
            Ok(name.to_string())
        }
    } else {
        Err(anyhow!("Could not find current local branch"))
    }
}

/// Return a tuple withe local and tracking remote branch configs, if present
/// stripping any remote prefixes (i.e. `origin/`)
fn get_current_branch(repo_path: &PathBuf) -> (Option<String>, Option<String>) {

    let local = get_current_local_branch_name(&repo_path).ok();

    let remote = if local.is_some() {
        get_current_remote_branch_name(&repo_path).ok()
    } else {
        None
    };

    debug!("(local, remote) = ({:?}, {:?})", local, remote);

    (local, remote)
}

fn branch_prefixed_with_issue_id(branch: &str, id: u64) -> bool {
    debug!("branch_prefixed_with_issue_id");
    branch.starts_with(&(id.to_string() + "-"))
}

fn create_remote_branch(p: u64, from: &str, branch: &str, gitlabclient: &Client) -> Result<String> {
    debug!("create_remote_branch");
    #[derive(Deserialize, Debug)]
    struct Branch { name: String }

    let mut b = GLCreateBranch::builder();
    let endpoint = b.project(p).ref_(from).branch(branch).build()
        .map_err(|e| anyhow!("Could not construct API call to create branch.\n {}",e))?;

    let branch: Branch = endpoint
        .query(gitlabclient)?;

    println!("Created remote branch {}", branch.name);
    Ok(branch.name)
}

fn slug(s: &str) -> String {
    debug!("slug");
    slugify!(s)
}

fn slug_and_prefix(id: u64, s: &str) -> String {
    debug!("slug_and_prefix");
    format!("{}-{}", id, slug(&s))
}

pub fn create_merge_request_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {

    // if not inside local repo error and exit
    config.repo_path.as_ref().ok_or_else(|| anyhow!("Local repo not found. Are you in the correct directory?"))?;

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;

    let (commit_head, commit_body) = get_commit_details(&config.repo_path.as_ref().unwrap())?;

    let defaultbranch = &config.defaultbranch.as_ref()
        .ok_or_else(|| anyhow!("Could not determine default remote branch - try `git lab project refresh`"))?;

    debug!("Default branch: {:#?}", defaultbranch);

    let (local_branch_name, remote_branch_name) = get_current_branch(&config.repo_path.as_ref().unwrap());

    debug!("Local branch name: {:#?}", local_branch_name);
    debug!("Remote branch name: {:#?}", remote_branch_name);

    let issue_arg = value_t!(args, "issue_id", u64).ok();

    debug!("Issue arg: {:#?}", issue_arg);

    let issue = if issue_arg.is_some() {
        #[derive(Deserialize, Debug)]
        struct Issue { iid: u64, title: String, state: String}
        let mut i = GLIssue::builder();
        let endpoint = generate_basic_issue_builder(&args, "issue_id", &config, &mut i)?;
        let issue: Issue = endpoint
            .query(&gitlabclient)
            .context("Failed to find issue")?;

        Some(issue)
    } else { None };

    let issue_title: Option<String>;

    if let Some(i) = issue {
        if i.state == "closed" {
            return Err(anyhow!(format!("Issue #{} is closed.", i.iid)))
        }
        issue_title = Some(i.title);
    } else {
        issue_title = None;
    }

    debug!("Issue title: {:#?}", issue_title);

    let interactive_title: String;

    let title = match (args.value_of("title"), issue_arg) {
        (Some(t), _) => Ok(t.to_string()),
        (_, Some(_)) => resolves_issue_mr_title(Ok(&issue_title.unwrap().as_str())),
        (None, None) => {
            if commit_head.is_some() && local_branch_name != Some(defaultbranch.to_string()) {
                interactive_title = Input::<String>::new()
                    .with_prompt("Title")
                    .allow_empty(false)
                    .with_initial_text(commit_head.unwrap())
                    .interact()?;
            } else {
                interactive_title = Input::<String>::new()
                    .with_prompt("Title")
                    .allow_empty(false)
                    .interact()?;
            }

            Ok(interactive_title)
        }
    }?;

    debug!("Title: {:#?}", title);

    let description = match (args.value_of("desc"), args.value_of("issue_id")) {
        (Some(d), Some(i)) => Some(d.to_string() + "\n\nCloses #" +  i),
        (None, Some(i)) => {
            if Confirm::new()
                    .with_prompt("Edit merge request description?")
                    .default(true)
                    .show_default(true)
                    .interact()?
            {
                match commit_body {
                    Some(body) => Editor::new()
                        .extension(".md")
                        .require_save(true)
                        .edit(&(body + &"\n\nCloses #".to_string() + i))?,
                    None => Editor::new()
                        .extension(".md")
                        .require_save(true)
                        .edit(&("<!-- insert MR description here - save and quit when done -->\n\nCloses #".to_string() + i))?,
                }
            } else {
                match commit_body {
                    Some(body) => Some(body + &"\n\nCloses #".to_string() + i),
                    None => Some("Closes #".to_string() + i),
                }
            }
        },
        (Some(d), None) => Some(d.to_string()),
        (None, None) => {
            if Confirm::new()
                    .with_prompt("Edit merge request description?")
                    .default(true)
                    .show_default(true)
                    .interact()?
            {
                match commit_body {
                    Some(body) => Editor::new()
                        .extension(".md")
                        .require_save(true)
                        .edit(&body)?,
                    None => Editor::new()
                        .extension(".md")
                        .require_save(true)
                        .edit("<!-- insert MR description here - save and quit when done -->")?,
                }
            } else {
                match commit_body {
                    Some(body) => Some(body),
                    None => None,
                }
            }
        },
    };

    debug!("Description: {:#?}", description);

    let target_branch = match (
        args.value_of("target_branch"),
        defaultbranch
    )
    {
        (Some(t), _) if remote_branch_exists(project_id, t, &gitlabclient) => Ok(t),
        (Some(t), _) => Err(anyhow!(format!(
            "Branch {} does not exist in the remote (GitLab), so cannot merge into it.",
            t
        ))),
        (None, _) => Ok(defaultbranch.as_str()),
    }?;

    debug!("Target branch: {:#?}", target_branch);

    let project_path = &config.path_with_namespace.unwrap();

    debug!("Project path: {:#?}", project_path);

    debug!("---- ({:#?}, {:#?}, {:#?}, {:#?}) ----",
        args.value_of("source_branch"),
        local_branch_name,
        remote_branch_name,
        issue_arg);

    let source_branch: String = match (
        args.value_of("source_branch"),
        local_branch_name,
        remote_branch_name,
        issue_arg
    ) {
        // Explicitly passed source branch

        (Some(s), _, _, None)
            if remote_branch_exists(project_id, s, &gitlabclient)
                && !open_mr_on_branch(project_path, s, &gitlabclient)
                =>
                {
                    debug!("1 Some({}) _ _ None", s);
                    Ok(s.to_string())
                }

        (Some(s), _, _, Some(i_id)) if !branch_prefixed_with_issue_id(s, i_id) =>

            Err(anyhow!(format!(
                "Passed branch {} must start with `{}-` to be associated with the issue.", s, i_id))),

        (Some(s), _, _, Some(i_id))
            if remote_branch_exists(project_id, s, &gitlabclient)
                && !open_mr_on_branch(project_path, s, &gitlabclient)
                && branch_prefixed_with_issue_id(s, i_id)
                =>
                {
                    debug!("2 Some({}) _ _ Some({})", s, i_id);
                    Ok(s.to_string())
                }

        (Some(s), _, _, _)
            if remote_branch_exists(project_id, s, &gitlabclient)
                && open_mr_on_branch(project_path, s, &gitlabclient)
                =>
            Err(anyhow!(format!(
                "Passed branch {} is already a source for an open merge request on the server.", s))),

        (Some(s), _, _, _)=> create_remote_branch(project_id, defaultbranch, s, &gitlabclient),

        // No source branch explicitly passed, so try to infer or create it using the tracking
        // remote branch

        // handle the case where an issue_id is passed
        (None, Some(_), Some(remote), Some(i_id))
            if remote_branch_exists(project_id, &remote, &gitlabclient)
                && branch_prefixed_with_issue_id(&remote, i_id) // assumed not to be master
                =>
                {
                    debug!("3 None Some(_) Some({}) Some({})", remote, i_id);
                    Ok(remote)
                }

        (None, Some(_), Some(remote), Some(i_id))
            if remote_branch_exists(project_id, &remote, &gitlabclient)
                && !branch_prefixed_with_issue_id(&remote, i_id)
                && &remote != *defaultbranch
                =>
                {
                    debug!("3a None Some(_) Some({}) None", remote);
                    Ok(remote)
                }

        // handle the case where a remote tracking branch exists
        (None, Some(_), Some(remote), None)
            if remote_branch_exists(project_id, &remote, &gitlabclient)
                // this implies that it's not the default (master) branch
                && open_mr_on_branch(project_path, &remote, &gitlabclient)
                =>
                Err(anyhow!(format!(
                    "Remote branch {} is already a source for an open merge request on the server.", remote))),

        (None, Some(_), Some(remote), None)
            if remote_branch_exists(project_id, &remote, &gitlabclient)
                && &remote != *defaultbranch
                =>
                {
                    debug!("4 None Some(_) Some({}) None", remote);
                    Ok(remote)
                }

        (None, Some(local), Some(remote), None)
            if remote_branch_exists(project_id, &remote, &gitlabclient)
                && &remote == *defaultbranch
                =>
                {
                    debug!("4a None Some(_) Some({}) None", remote);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &slug(&title), &gitlabclient)
                }

        // handle the case where a remote tracking branch is present locally but does not exist on
        // the server, probably because it was deleted on the server
        (None, Some(_), Some(remote), Some(i_id))
            if !remote_branch_exists(project_id, &remote, &gitlabclient)
                // && branch_prefixed_with_issue_id(&remote, i_id)
                =>
                {
                    debug!("5 None Some(_) Some({}) Some({})", remote, i_id);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &remote, &gitlabclient)
                }

        (None, Some(_), Some(remote), None)
            if !remote_branch_exists(project_id, &remote, &gitlabclient)
                =>
                {
                    debug!("6 None Some(_) Some({}) None)", remote);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &remote, &gitlabclient)
                }

        // No source branch explicitly passed, so try to infer or create it using the local branch,
        // as no tracking remote appears to be set up
        (None, Some(local), None, Some(i_id))
            if branch_prefixed_with_issue_id(&local, i_id)
                =>
                {
                    debug!("7 None Some({}) None None)", local);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &local, &gitlabclient)
                }

        (None, Some(local), None, Some(i_id))
            if &local != *defaultbranch
                =>
            Err(anyhow!(format!(
                "Local branch {} must start with `{}-` to be associated with the issue.", &local, i_id))),

        (None, Some(local), _, Some(i_id))
            if &local == *defaultbranch
               && remote_branch_exists(project_id, &slug_and_prefix(i_id, &title), &gitlabclient)
                =>
            Err(anyhow!(format!(
                "Remote branch {} exists on the server and is already associated to issue #{}",
                &slug_and_prefix(i_id, &title), i_id))),

        (None, Some(local), _, Some(i_id))
            if &local == *defaultbranch
               && !remote_branch_exists(project_id, &slug_and_prefix(i_id, &title), &gitlabclient)
                =>
                {
                    debug!("8 None Some({}) None Some({})", local, i_id);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &slug_and_prefix(i_id, &title), &gitlabclient)
                }

        (None, Some(local), None, None)
            if &local != *defaultbranch
               && !remote_branch_exists(project_id, &local, &gitlabclient)
                =>
                {
                    debug!("9 None Some({}) None None", local);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &local, &gitlabclient)
                }

        // no explicit source branch or issue created, and on the master branch,
        // so create the source branch from the title
        (None, Some(local), _, None)
            if &local == *defaultbranch
               && !remote_branch_exists(project_id, &slug(&title), &gitlabclient)
                =>
                {
                    debug!("10 None Some({}) None None", local);
                    debug!("Creating remote branch...");
                    create_remote_branch(project_id, defaultbranch, &slug(&title), &gitlabclient)
                }

        (s, l, r, i)
            =>
            {
                debug!("---- ({:#?}, {:#?}, {:#?}, {:#?}) ----", s, l, r, i);
                unreachable!()
            }
    }?;

    debug!("Source branch: {:#?}", source_branch);

    let mut mr = CreateMergeRequest::builder();
    let endpoint = mr
        .project(project_id)
        .target_branch(target_branch)
        .source_branch(&source_branch)
        .title("WIP: ".to_string() + &title);

    if let Some(d) = description {
        endpoint.description(d);
    };

    if args.occurrences_of("squash") > 0 {
        endpoint.squash(true);
    };

    if args.occurrences_of("remove_src") > 0 {
        endpoint.remove_source_branch(true);
    };

    let endpoint = endpoint
        .build()
        .map_err(|e| anyhow!("Could not construct API call to create merge request.\n {}",e))?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    let merge_request: MergeRequest = endpoint
        .query(&gitlabclient)
        .context("Failed to create merge request")?;

    println!("Merge Request created at: {}", merge_request.web_url);

    if args.occurrences_of("checkout") > 0 {
        let fetch = Command::new("git")
            .args(&["fetch","origin"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        // println!("{}",fetch);

        let checkout = Command::new("git")
            .args(&["checkout","-b", &source_branch, &("origin/".to_string() + &source_branch)])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        // println!("{}",checkout);
    }

    Ok(())
}
