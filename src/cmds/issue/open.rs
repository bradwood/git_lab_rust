use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::config;
use crate::gitlab::{Client, IssueBuilder, Query};
use crate::gitlab::Issue as GLIssue;
use crate::utils;

#[derive(Debug, Deserialize)]
struct Issue {
    web_url: String,
}

pub fn generate_issue_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    i: &'a mut IssueBuilder<'a>,
) -> Result<GLIssue<'a>> {

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    i.project(project_id);
    i.issue(args.value_of("id").unwrap().parse::<u64>().unwrap());
    i.build()
        .map_err(|e| anyhow!("Could not construct query to fetch project URL from server.\n {}",e))
}

pub fn open_issue_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLIssue::builder();
    let endpoint = generate_issue_builder(&args, &config, &mut p)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    let issue: Issue = endpoint
        .query(&gitlabclient)
        .context("Failed to find issue")?;

    match args.occurrences_of("url") {
        1u64..=std::u64::MAX => {
            let out_vars = vec!(("web_url".to_string(), issue.web_url)).into_iter();
            utils::write_short_output(config.format, out_vars)
        },

        0  => {
            match webbrowser::open(&issue.web_url) {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow!("Could not open URL. Try setting BROWSER."))
            }
        },
    }
}

