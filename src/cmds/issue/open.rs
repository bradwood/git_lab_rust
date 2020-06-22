use anyhow::{anyhow, Context, Result};

use crate::cmds::issue::{generate_basic_issue_builder, Issue};
use crate::config;
use crate::gitlab::{Client, Query};
use crate::gitlab::Issue as GLIssue;
use crate::utils;

pub fn open_issue_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLIssue::builder();
    let endpoint = generate_basic_issue_builder(&args, &config, &mut p)?;

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

