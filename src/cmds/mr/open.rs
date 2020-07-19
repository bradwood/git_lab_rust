use anyhow::{anyhow, Context, Result};

use crate::cmds::mr::{generate_basic_mr_builder, MergeRequest};
use crate::config;
use crate::gitlab::{Client, Query};
use crate::gitlab::MergeRequest as GLMergeRequest;
use crate::utils;

pub fn open_merge_request_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLMergeRequest::builder();
    let endpoint = generate_basic_mr_builder(&args, "id", &config, &mut p)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    let mr: MergeRequest = endpoint
        .query(&gitlabclient)
        .context("Failed to find merge request")?;

    match args.occurrences_of("url") {
        1u64..=std::u64::MAX => {
            let out_vars = vec!(("web_url".to_string(), mr.web_url)).into_iter();
            utils::write_short_output(config.format, out_vars)
        },

        0  => {
            match webbrowser::open(&mr.web_url) {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow!("Could not open URL. Try setting BROWSER."))
            }
        },
    }
}

