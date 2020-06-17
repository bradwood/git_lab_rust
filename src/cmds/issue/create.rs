
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use clap::value_t_or_exit;
use dialoguer::{Input, Editor};
use serde::Deserialize;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::{api, Client, CreateIssue, CreateIssueBuilder, Query};
use crate::utils;
use crate::utils::validator;

#[derive(Debug, Deserialize)]
struct Issue {
    id: u64,
    web_url: String,
}

pub fn generate_issue_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    i: &'a mut CreateIssueBuilder<'a>,
) -> Result<CreateIssue<'a>> {

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    i.project(project_id);

    for arg in &args.args {
        let (key, _) = arg;
        match *key {
            // straight string arguments
            "title" => i.title(args.value_of("title").unwrap()),
            "description" => i.description(args.value_of("description").unwrap()),

            // u64 arguments
            "project_id" => i.project(value_t_or_exit!(args, "project_id", u64)),
            "milestone_id" => i.milestone_id(value_t_or_exit!(args, "milestone_id", u64)),
            "weight" => i.weight(value_t_or_exit!(args, "weight", u64)),

            // boolean flags
            "confidential" => i.confidential(true),

            // date flags
            "due_date" => i.due_date(
                NaiveDate::parse_from_str(args.value_of("due_date").unwrap(), "%Y-%m-%d")
                .unwrap()
                ),

            // list parameters
            "labels" => i.labels(args.values_of("labels").unwrap()),

            // TODO add assignees
            // "assignees" => i.assignee_ids(args.values_of("assignees").unwrap()),

            _ => unreachable!(),
        };
    }

    i.build()
        .map_err(|e| anyhow!("Could not construct query to post issue to server.\n {}",e))
}

fn interactive_issue_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    i: &'a mut CreateIssueBuilder<'a>,
) -> Result<CreateIssue<'a>> {

    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    i.project(project_id);

    let title = Input::<String>::new()
        .with_prompt("Title")
        .interact()?;
    i.title(title);

    let description = Editor::new()
        .extension(".md")
        .require_save(true)
        .edit("Issue description (markdown supported)")?;

    if let Some(desc) = description {
        i.description(desc);
    }

    #[allow(clippy::redundant_closure)]  // below closure doesn't work unless called as shown below
    let weight = Input::<String>::new()
        .with_prompt("Weight")
        .allow_empty(true)
        .validate_with(|d: &str| validator::check_u32_or_empty(d))
        .interact()?;

    if !weight.is_empty() {
        i.weight(
            weight.parse::<u64>()
            .unwrap()
        );
    }

    let confidential = Input::<bool>::new()
        .with_prompt("Confidential")
        .default(false)
        .interact()?;
    i.confidential(confidential);

    #[allow(clippy::redundant_closure)]  // below closure doesn't work unless called as shown below
    let due_date = Input::<String>::new()
        .with_prompt("Due date [YYYY-MM-DD]")
        .allow_empty(true)
        .validate_with(|d: &str| validator::check_yyyy_mm_dd_or_empty(d))
        .interact()?;

    if !due_date.is_empty() {
        i.due_date(
            NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
            .unwrap()
        );
    }

    //TODO: add assignee, labels and milestone selectors

    i.build()
        .map_err(|e| anyhow!("Could not construct query to post issue to server.\n {}",e))
}

pub fn create_issue_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut i = CreateIssue::builder();

    let interactive = !args.is_present("title");

    let endpoint = if !interactive {
        generate_issue_builder(&args, &config, &mut i)?
    } else {
        interactive_issue_builder(&args, &config, &mut i)?
    };

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match (&config.format, interactive) {

        (_, true) | (Some(OutputFormat::Text), _) => {
            let issue: Issue = endpoint
                .query(&gitlabclient)
                .context("Failed to create issue")?;

            println!("Issue id: {}", issue.id);
            println!("Issue URL: {}", issue.web_url);
            Ok(())
        },

        (Some(OutputFormat::JSON), _) => {
            let raw_json  = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to create issue")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        },

        (None, _) => Err(anyhow!("Bad output format in config")),
    }
}

