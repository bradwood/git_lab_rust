use anyhow::{Context, Result};

use crate::config;
use crate::gitlab::{CreateProjectParams, IfGitLab};

pub fn create_project_cmd(
    config: config::Config,
    args: clap::ArgMatches,
    gitlab: impl IfGitLab,
) -> Result<()> {
    trace!("Config: {:?}", config);
    trace!("Args: {:?}", args);

    let mut params = &mut CreateProjectParams::builder();

    if args.is_present("description") {
        params = params.description(args.value_of("description").unwrap());
        trace!("description: {}", args.value_of("description").unwrap());
    }

    // .description("Splendid project")
    // // set up XDG and Global configs
    // .build()
    // .unwrap();

    // TODO: Consider changing return value to Result<serde_json::Value> to get raw json.
    let project = gitlab.create_project(
        args.value_of("name").unwrap(),
        args.value_of("path"),
        Some(params.build().unwrap()),
    ).context("Failed to create project - check for name or path clashes on the server")?;

    println!("Project id: {}", project.id);
    println!("Project URL: {}", project.web_url);
    Ok(())
}

#[cfg(test)]
mod project_create_unit_tests {


}
