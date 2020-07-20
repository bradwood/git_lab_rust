use anyhow::{anyhow, Context, Result};
use clap::value_t_or_exit;

use crate::config;
use crate::gitlab::{api, Client, EditMergeRequest, Query, MergeRequestStateEvent};
use crate::utils;
use crate::utils::ShortCmd;

pub fn quick_edit_mr_cmd(
    args: clap::ArgMatches,
    shortcmd: ShortCmd,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut m = EditMergeRequest::builder();

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    m.project(project_id);

    let mr_id = value_t_or_exit!(args, "id", u64);
    m.merge_request(mr_id);

    match shortcmd {
        ShortCmd::Close => m.state_event(MergeRequestStateEvent::Close),
        ShortCmd::Reopen => m.state_event(MergeRequestStateEvent::Reopen),
        ShortCmd::Lock => m.discussion_locked(true),
        ShortCmd::Unlock => m.discussion_locked(false),
        ShortCmd::Assign => {
            let assign_ids = utils::map_user_ids_from_names(&config.members, args.values_of("usernames").unwrap())?;
            m.assignees(assign_ids.into_iter())
        }
    };

    let endpoint = m
        .build()
        .map_err(|e| anyhow!("Could not construct edit query.\n{}", e))?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    api::ignore(endpoint)
        .query(&gitlabclient)
        .context("Failed to update merge request")?;

    Ok(())
}
