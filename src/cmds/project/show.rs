use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::{api, Client, Query};
use crate::gitlab::Project as GLProject;
use crate::cmds::project::open;

#[derive(Debug, Deserialize)]
struct Project {
    id: u64,
    owner: Map<String, Value>,
    web_url: String,
    created_at: String,
    ssh_url_to_repo: String,
    http_url_to_repo: String,
    forks_count: u64,
    star_count: u64,
    visibility: String,
}

pub fn show_project_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLProject::builder();
    let endpoint = open::generate_project_builder(&args, &config, &mut p)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json  = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to find project")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        },

        Some(OutputFormat::Text) => {
            let project: Project = endpoint
                .query(&gitlabclient)
                .context("Failed to find project")?;

            println!("ID: {}", project.id);
            println!("Owner: {}", project.owner["name"].as_str().unwrap());
            println!("Owner's URL: {}", project.owner["web_url"].as_str().unwrap());
            println!("Created: {}",
                DateTime::parse_from_rfc3339(
                    project.created_at.as_str())
                    .unwrap()
                    .with_timezone(&Local)
                    .to_rfc2822()
                );
            println!("Web URL: {}", project.web_url);
            println!("SSH Repo URL: {}", project.ssh_url_to_repo);
            println!("HTTP Repo URL: {}", project.http_url_to_repo);
            println!("Stars: {}", project.star_count);
            println!("Forks: {}", project.forks_count);
            println!("Visibility: {}", project.visibility);
            Ok(())
        },
        _ => Err(anyhow!("Bad output format in config")),
    }
}

