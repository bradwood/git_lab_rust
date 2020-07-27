use anyhow::{anyhow, Context, Result};
use clap::value_t_or_exit;

use crate::config;
use crate::gitlab::{api, Client, ApproveMergeRequest, Query};
use crate::utils;


pub fn approve_mr_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut m = ApproveMergeRequest::builder();

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    m.project(project_id);

    let mr_id = value_t_or_exit!(args, "id", u64);
    m.merge_request(mr_id);

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
