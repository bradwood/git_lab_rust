use anyhow::Result;

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

    gitlab.create_project(
        // args.value_of("name").unwrap(),
        "testproj",
        Some("the_path"),
        None
        // args.value_of("path"),
        // Some(params.build().unwrap()),
    )?;
    trace!("created project: {}", args.value_of("name").unwrap());
    Ok(())
}


// TODO: add test for create object that passes in a mock gitlab object
