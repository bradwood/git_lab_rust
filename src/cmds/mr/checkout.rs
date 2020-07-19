use anyhow::{Context, Result};

use crate::cmds::mr::{checkout_mr, generate_basic_mr_builder, MergeRequest};
use crate::config;
use crate::gitlab::{Client, Query};
use crate::gitlab::MergeRequest as GLMergeRequest;

pub fn checkout_merge_request_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLMergeRequest::builder();
    let endpoint = generate_basic_mr_builder(&args, "id", &config, &mut p)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    let mr: MergeRequest = endpoint
        .query(&gitlabclient)
        .context("Failed to find merge request")?;

    checkout_mr(&mr.source_branch)?;
    Ok(())
}

