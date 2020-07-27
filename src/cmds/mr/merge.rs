use anyhow::{anyhow, Context, Result};
use clap::value_t_or_exit;

use crate::config;
use crate::gitlab::{api, Client, MergeMergeRequest, Query};
use crate::utils;

pub fn merge_mr_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut m = MergeMergeRequest::builder();

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    m.project(project_id);

    let mr_id = value_t_or_exit!(args, "id", u64);
    m.merge_request(mr_id);

    if args.occurrences_of("merge_when_pipeline_succeeds") > 0 {
        m.merge_when_pipeline_succeeds(true);
    }

    if args.occurrences_of("dont_del_source_branch") == 0 {
        m.should_remove_source_branch(true);
    }

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
