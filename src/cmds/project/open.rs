use anyhow::{anyhow, Context, Result};

use crate::cmds::project::{generate_basic_project_builder, Project};
use crate::config;
use crate::gitlab::Project as GLProject;
use crate::gitlab::{Client, Query};
use crate::utils;

pub fn open_project_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLProject::builder();
    let endpoint = generate_basic_project_builder(&args, &config, &mut p)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    let project: Project = endpoint
        .query(&gitlabclient)
        .context("Failed to find project")?;

    match args.occurrences_of("url") {
        1u64..=std::u64::MAX => {
            let out_vars = vec!(("web_url".to_string(), project.web_url)).into_iter();
            utils::write_short_output(config.format, out_vars)
        },

        0  => {
            match webbrowser::open(&project.web_url) {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow!("Could not open URL. Try setting BROWSER."))
            }
        },
    }
}

