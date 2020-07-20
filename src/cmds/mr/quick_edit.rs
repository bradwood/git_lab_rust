use anyhow::{anyhow, Context, Result};
use clap::value_t_or_exit;
use crate::cmds::mr::{generate_basic_mr_builder, MergeRequest};

use crate::config;
use crate::gitlab::{api, Client, EditMergeRequest, Query, MergeRequestStateEvent};
use crate::gitlab::MergeRequest as GLMergeRequest;
use crate::utils;
use crate::utils::ShortCmd;

fn strip_wip(s: String) -> String {
    for prefix in ["[WIP]","WIP:", "Draft:", "[Draft]", "(Draft)"].iter() {
        if s.starts_with(prefix) {
            return s.strip_prefix(prefix).unwrap().to_string();
        }
    }
    s

}

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
        ShortCmd::Wip => {
            let mut p = GLMergeRequest::builder();
            let endpoint = generate_basic_mr_builder(&args, "id", &config, &mut p)?;
            let mr: MergeRequest = endpoint
                .query(&gitlabclient)
                .context("Failed to find merge request")?;

            match (mr.work_in_progress, args.is_present("on"), args.is_present("off")) {
                (true, _, true) => {
                    // turn off WIP,
                    m.title(strip_wip(mr.title))
                }
                (false, true, _) => {
                    // turn on WIP,
                    m.title("WIP: ".to_string() + &mr.title)
                }
                _ => m.title(mr.title) // do nothing,
            }
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
