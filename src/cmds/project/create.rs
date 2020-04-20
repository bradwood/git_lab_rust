use anyhow::Result;

use crate::config;
use crate::gitlab::{gen_gitlab, CreateProjectParams, GitLabShim};

pub fn create_project(config: config::Config, args: clap::ArgMatches) -> Result<()> {
    trace!("Config: {:?}", config);
    trace!("Args: {:?}", args);

    let gitlab = gen_gitlab(config)?;
    let mut params = &mut CreateProjectParams::builder();

    if args.is_present("description") {
        params = params.description(args.value_of("description").unwrap());
        trace!("description: {}", args.value_of("description").unwrap());
    }

    // .description("Splendid project")
    // .build()
    // .unwrap();

    gitlab.create_project(
        args.value_of("name").unwrap(),
        Some(args.value_of("path").unwrap()),
        Some(params.build().unwrap()),
    )?;
    trace!("created project: {}", args.value_of("name").unwrap());
    Ok(())
}
