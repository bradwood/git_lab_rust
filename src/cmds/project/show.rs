use anyhow::{anyhow, Context, Result};
use chrono::{Local};

use crate::cmds::project::{generate_basic_project_builder, Project};
use crate::config::OutputFormat;
use crate::config;
use crate::gitlab::Project as GLProject;
use crate::gitlab::{api, Client, Query};

fn print_project(p: Project) {
    println!("ID: {}", p.id);
    if let Some(o) = p.owner {
        println!("Owner: {}", o["name"].as_str().unwrap());
        println!("Owner's URL: {}", o["web_url"].as_str().unwrap());
    }
    println!("Created: {}", p.created_at.with_timezone(&Local).to_rfc2822());
    println!("Web URL: {}", p.web_url);
    println!("SSH Repo URL: {}", p.ssh_url_to_repo);
    println!("HTTP Repo URL: {}", p.http_url_to_repo);
    println!("Stars: {}", p.star_count);
    println!("Forks: {}", p.forks_count);
    println!("Visibility: {}", p.visibility);
}

pub fn show_project_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLProject::builder();
    let endpoint = generate_basic_project_builder(&args, &config, &mut p)?;

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

            print_project(project);
            Ok(())
        },
        _ => Err(anyhow!("Bad output format in config")),
    }
}

