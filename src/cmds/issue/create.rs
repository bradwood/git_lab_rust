
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use clap::value_t_or_exit;
use serde::Deserialize;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::{api, Client, CreateIssue, CreateIssueBuilder, Query};

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

    match (config.projectid, args.value_of("project_id")) {
        (None, Some(a_id)) => i.project(a_id),
        (Some(c_id), None) => i.project(c_id),
        (Some(_), Some(a_id)) => i.project(a_id),
        (None, None) =>
            return Err(anyhow!("No project ID passed and project not attached to the current repo. Run `git lab project attach`"))
    };

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
            // "assignees" => i.assignee_ids(args.values_of("assignees").unwrap()),

            _ => unreachable!(),
        };
    }

    i.build()
        .map_err(|e| anyhow!("Could not construct query to post issue to server.\n {}",e))
}

pub fn create_issue_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut i = CreateIssue::builder();
    let endpoint = generate_issue_builder(&args, &config, &mut i)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json  = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to create issue")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        },

        Some(OutputFormat::Text) => {
            let issue: Issue = endpoint
                .query(&gitlabclient)
                .context("Failed to create issue")?;

            println!("Issue id: {}", issue.id);
            println!("Issue URL: {}", issue.web_url);
            Ok(())
        },
        _ => Err(anyhow!("Bad output format in config")),
    }
}

