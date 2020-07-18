use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use clap::value_t_or_exit;
use comfy_table::*;
use serde::Deserialize;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::converter::{
    issue_order_by_from_str, issue_scope_from_str, issue_state_from_str,
};
use crate::gitlab::{api, Client, IssueWeight, Issues, IssuesBuilder, Query, SortOrder};
use crate::utils;

#[derive(Debug, Deserialize)]
pub struct Issue {
    iid: u64,
    title: String,
    state: String,
    created_at: DateTime<Utc>,
}

pub fn generate_issues_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    i: &'a mut IssuesBuilder<'a>,
) -> Result<Issues<'a>> {
    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    i.project(project_id);

    for arg in &args.args {
        let (key, _) = arg;
        match *key {
            "state" if args.value_of("state").unwrap() != "all" => {
                i.state(issue_state_from_str(args.value_of("state").unwrap()).unwrap())
            }
            "state" if args.value_of("state").unwrap() == "all" => i,
            "scope" => i.scope(issue_scope_from_str(args.value_of("scope").unwrap()).unwrap()),
            "labels" => i.labels(args.values_of("labels").unwrap()),
            "unlabelled" => i.unlabeled(),
            "labelled" => i.with_any_label(),
            "author" => i.author(args.value_of("author").unwrap()),
            "assignees" => i.assignees(args.values_of("assignees").unwrap()),
            "assigned" => i.assigned(),
            "unassigned" => i.unassigned(),
            "weight" => i.weight(IssueWeight::Weight(value_t_or_exit!(args, "weight", u64))),
            "weighted" => i.weight(IssueWeight::Any),
            "unweighted" => i.weight(IssueWeight::None),
            "filter" => i.search(args.value_of("filter").unwrap()),
            "created_after" => i.created_after(datefield!("created_after", args)),
            "created_before" => i.created_before(datefield!("created_before", args)),
            "updated_after" => i.updated_after(datefield!("updated_after", args)),
            "updated_before" => i.updated_before(datefield!("updated_before", args)),
            "confidential" => i.confidential(true),
            "order_by" => {
                i.order_by(issue_order_by_from_str(args.value_of("order_by").unwrap()).unwrap())
            }
            "descending" => i.sort(SortOrder::Descending),
            "ascending" => i.sort(SortOrder::Ascending),
            "max" => i,
            _ => unreachable!(),
        };
    }
    i.build()
        .map_err(|e| anyhow!("Could not construct issues query.\n {}", e))
}

fn print_issues(issues: Vec<Issue>) {
    let mut table = Table::new();
    table
        .load_preset("                   ")
        .set_content_arrangement(ContentArrangement::Dynamic);

    for i in issues {
        let create_date  = format!("{}", HumanTime::from(i.created_at));

        let id = if i.state == "opened" {
            Cell::new(i.iid).add_attribute(Attribute::Bold).fg(Color::Yellow)
        } else {
            Cell::new(i.iid).add_attribute(Attribute::Dim)
        };

        let title = if i.state == "opened" {
            Cell::new(i.title).add_attribute(Attribute::Bold)
        } else {
            Cell::new(i.title).add_attribute(Attribute::Dim)
        };

        table.add_row(vec![
            id,
            title,
            Cell::new("about ".to_string() + &create_date).add_attribute(Attribute::Dim),
        ]);
    }
    println!("{}", table);
}


pub fn list_issues_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut i = Issues::builder();
    let endpoint = generate_issues_builder(&args, &config, &mut i)?;
    let max = value_t_or_exit!(args, "max", u32);

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to query issues")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        }

        Some(OutputFormat::Text) => {
            let issues: Vec<Issue> = api::paged(endpoint, api::Pagination::Limit(max as usize))
                .query(&gitlabclient)
                .context("Failed to query issues")?;

            print_issues(issues);

            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
