use anyhow::{anyhow, Context, Result};
use clap::value_t_or_exit;

use crate::config;
use crate::gitlab::{api, Client, EditIssue, Query, IssueStateEvent};
use crate::utils;
use crate::utils::ShortCmd;

pub fn quick_edit_issue_cmd(
    args: clap::ArgMatches,
    shortcmd: ShortCmd,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut i = EditIssue::builder();

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    i.project(project_id);

    let issue_id = value_t_or_exit!(args, "id", u64);
    i.issue(issue_id);

    match shortcmd {
        ShortCmd::Close => i.state_event(IssueStateEvent::Close),
        ShortCmd::Reopen => i.state_event(IssueStateEvent::Reopen),
        ShortCmd::Lock => i.discussion_locked(true),
        ShortCmd::Unlock => i.discussion_locked(false),
        ShortCmd::Assign => {
            let assign_ids = utils::map_user_ids_from_names(&config.members, args.values_of("usernames").unwrap())?;
            i.assignee_ids(assign_ids.into_iter())
        }
        ShortCmd::Wip => unreachable!()
    };

    let endpoint = i
        .build()
        .map_err(|e| anyhow!("Could not construct edit query.\n{}", e))?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    api::ignore(endpoint)
        .query(&gitlabclient)
        .context("Failed to update issue")?;

    Ok(())
}
