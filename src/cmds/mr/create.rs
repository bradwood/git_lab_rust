use anyhow::{anyhow, Result};
use clap::value_t;
use dialoguer::Input;

// use crate::cmds::issue::Issue;
use crate::config;
// use crate::config::OutputFormat;
use crate::gitlab::{api, Client, CreateMergeRequest, CreateMergeRequestBuilder, Query};
use crate::utils;
// use crate::utils::validator;

fn resolves_issue_mr_title(p: u64, i: u64, gitlabclient: &Client) -> Result<&'static str> {
    Ok(&"Resolves some issue title that is still to be implemented")
}

fn get_default_remote_branch(p: u64, gitlabclient: &Client) -> &'static str {
    //TODO: cache the answer to this in config
    &"master"
}

fn remote_branch_exists(p: u64, branch: &str, gitlabclient: &Client) -> bool {
    true
}

fn open_mr_on_branch(p: u64, branch: &str, gitlabclient: &Client) -> bool {
    false
}

fn is_default_branch(branch: &str) -> bool {
    false
}

fn get_current_branch() -> (Option<&'static str>, Option<&'static str>) {
    (Some(&"local_branch_name"), Some(&"remote_branch_name"))
}

fn branch_prefixed_with_issue_id(branch: &str, id: u64) -> bool {
    true
}

fn create_remote_branch(project_id: u64, branch: &str, gitlabclient: &Client) -> Result<&'static str> {

    Ok(&"branch_name")
}

fn slugify(s: &str) -> &str {
    &"slug-if-fied-branch-name"
}

fn slugify_and_prefix(id: u64, s: &str) -> String {
    format!("{}-{}", id, s)
}

pub fn create_merge_request_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut mr = CreateMergeRequest::builder();

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    mr.project(project_id);

    let issue_arg = value_t!(args, "issue_id", u64).ok();

    let interactive_title: String;

    let title = match (args.value_of("title"), issue_arg) {
        (Some(t), _) => Ok(t),
        (_, Some(i)) => resolves_issue_mr_title(project_id, i, &gitlabclient),
        (None, None) =>
            //TODO Infer initial text from the HEAD's commit message?? Add desc if multi-line commit message...
        {
            interactive_title = Input::<String>::new()
            .with_prompt("Title")
            .allow_empty(false)
            // .with_initial_text("TODO - get first line of commit message")
            .interact()?;

            Ok(interactive_title.as_str())
        }
    }?;

    let target_branch = match (
        args.value_of("target_branch"),
        get_default_remote_branch(project_id, &gitlabclient),
    ) {
        (Some(t), _) if remote_branch_exists(project_id, t, &gitlabclient) => Ok(t),
        (Some(t), _) => Err(anyhow!(format!(
            "Branch {} does not exist in the remote (GitLab), so cannot merge into it.",
            t
        ))),
        (None, m) => Ok(m),
    }?;

    let (local_branch_name, remote_branch_name) = get_current_branch();

    let source_branch: &str = match (
        args.value_of("source_branch"),
        local_branch_name,
        remote_branch_name,
        issue_arg
    ) {
        // Explicitly passed source branch

        (Some(s), _, _, None)
            if remote_branch_exists(project_id, s, &gitlabclient)
                && !open_mr_on_branch(project_id, s, &gitlabclient) // TODO: check for master branch?
                =>

            Ok(s),

        (Some(s), _, _, Some(i_id)) if !branch_prefixed_with_issue_id(s, i_id) =>

            Err(anyhow!(format!(
                "Passed branch {} must start with `{}-` to be associated with the issue.", s, i_id))),

        (Some(s), _, _, Some(i_id))
            if remote_branch_exists(project_id, s, &gitlabclient)
                && !open_mr_on_branch(project_id, s, &gitlabclient)
                && branch_prefixed_with_issue_id(s, i_id)
                =>

            Ok(s),

        (Some(s), _, _, _)
            if remote_branch_exists(project_id, s, &gitlabclient)
                && open_mr_on_branch(project_id, s, &gitlabclient)
                =>
            Err(anyhow!(format!(
                "Passed branch {} is already a source for an open merge request on the server.", s))),

        (Some(s), _, _, _) => create_remote_branch(project_id, s, &gitlabclient),

        // No source branch explicitly passed, so try to infer or create it using the tracking
        // remote branch

        // handle the case where an issue_id is passed
        (None, Some(_), Some(remote), Some(i_id))
            if remote_branch_exists(project_id, remote, &gitlabclient)
                && branch_prefixed_with_issue_id(remote, i_id)
                =>
            Ok(remote),

        (None, Some(_), Some(remote), Some(i_id))
            if remote_branch_exists(project_id, remote, &gitlabclient)
                && !branch_prefixed_with_issue_id(remote, i_id)
                && !is_default_branch(remote)
                =>
            Err(anyhow!(format!(
                "Remote branch {} must start with `{}-` to be associated with the issue.", remote, i_id))),

        // handle the case where a remote tracking branch exists
        (None, Some(_), Some(remote), None)
            if remote_branch_exists(project_id, remote, &gitlabclient)
                && !is_default_branch(remote)
                =>
            Ok(remote),

        (None, Some(_), Some(remote), None)
            if remote_branch_exists(project_id, remote, &gitlabclient)
                && open_mr_on_branch(project_id, remote, &gitlabclient) // this implies that it's not the default (master) branch
                =>
            Err(anyhow!(format!(
                "Remote branch {} is already a source for an open merge request on the server.", remote))),

        // handle the case where a remote tracking branch is present locally but does not exist on
        // the server, probably because it was deleted on the server
        (None, Some(_), Some(remote), Some(i_id))
            if !remote_branch_exists(project_id, remote, &gitlabclient)
                && branch_prefixed_with_issue_id(remote, i_id)
                =>
            create_remote_branch(project_id, remote, &gitlabclient),

        (None, Some(_), Some(remote), Some(i_id))
            if !remote_branch_exists(project_id, remote, &gitlabclient)
                && !branch_prefixed_with_issue_id(remote, i_id)
                =>
            Err(anyhow!(format!(
                "Remote branch {} must start with `{}-` to be associated with the issue.", remote, i_id))),

        (None, Some(_), Some(remote), None)
            if !remote_branch_exists(project_id, remote, &gitlabclient) =>
            create_remote_branch(project_id, remote, &gitlabclient),

        // No source branch explicitly passed, so try to infer or create it using the local branch,
        // as no tracking remote appears to be set up

        //TODO: should this also configure the local to track this new remote branch???
        (None, Some(local), None, Some(i_id))
            if branch_prefixed_with_issue_id(local, i_id) =>
            create_remote_branch(project_id, local, &gitlabclient),

        (None, Some(local), None, Some(i_id))
            if !is_default_branch(local) =>
            Err(anyhow!(format!(
                "Local branch {} must start with `{}-` to be associated with the issue.", local, i_id))),

        (None, Some(local), None, Some(i_id))
            if is_default_branch(local) =>
            create_remote_branch(project_id, &slugify_and_prefix(i_id, title), &gitlabclient),

        (None, Some(local), None, None)
            if !is_default_branch(local) =>
            create_remote_branch(project_id, local, &gitlabclient),

        (None, Some(local), None, None)
            if
                is_default_branch(local)
                =>
            create_remote_branch(project_id, slugify(title), &gitlabclient),

        (_, _, _, _) => unreachable!(),

    }?;

    Ok(())
}
