use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::config;
use crate::gitlab::{Client, ProjectBuilder, Query};
use crate::gitlab::Project as GLProject;
use crate::utils;

#[derive(Debug, Deserialize)]
struct Project {
    web_url: String,
}

pub fn generate_project_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    p: &'a mut ProjectBuilder<'a>,
) -> Result<GLProject<'a>> {

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    p.project(project_id);
    p.build()
        .map_err(|e| anyhow!("Could not construct query to fetch project URL from server.\n {}",e))
}

pub fn open_project_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLProject::builder();
    let endpoint = generate_project_builder(&args, &config, &mut p)?;

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

