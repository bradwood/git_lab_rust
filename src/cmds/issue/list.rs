use anyhow::{anyhow, Context, Result};
use chrono::{Utc, DateTime, Local};
use chrono_humanize::HumanTime;
use clap::{value_t_or_exit, values_t_or_exit};
use comfy_table::*;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::converter::{
    issue_order_by_from_str, issue_scope_from_str, issue_state_from_str,
};
use crate::gitlab::{api, Client, IssueWeight, Issues, IssuesBuilder, Query, SortOrder};
use crate::utils;
use crate::cmds::issue::Issue;

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
            // presentational arguments
            "max" => i,
            "fields" => i,
            "no_headers" => i,
            "human_friendly" => i,
            _ => unreachable!(),
        };
    }
    i.build()
        .map_err(|e| anyhow!("Could not construct issues query.\n {}", e))
}

fn print_issues(issues: Vec<Issue>, fields: Vec<String>, no_headers: bool, human: bool) {
    let mut table = Table::new();

    table
        .load_preset("                   ")
        .set_content_arrangement(ContentArrangement::Dynamic);

    if !no_headers {
        table.add_row(fields.iter().map(|f| Cell::new(f.to_uppercase().replace("_"," ")).set_alignment(CellAlignment::Center)));
    }

    for i in issues {
        let mut r: Vec<Cell> =Vec::new();

        for field in &fields {
            match field.as_str() {
                "assignees" => {
                    if i.assignees.is_some() {
                        r.push(
                            Cell::new(
                                i.assignees.clone()
                                .unwrap()
                                .iter()
                                .map(|a| a["username"].as_str().unwrap().to_string())
                                .collect::<Vec<String>>().join(",")
                            )
                        )
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "author" => r.push(Cell::new(i.author["username"].as_str().unwrap())),
                "closed_by" => {
                    if i.closed_by.is_some() {
                        r.push(Cell::new(i.closed_by.clone().unwrap()["username"].as_str().unwrap()))
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "closed_on" => {
                    if i.closed_at.is_some() {
                        if human {
                            r.push(Cell::new(HumanTime::from(i.closed_at.unwrap())))
                        } else {
                            let d: DateTime<Local> = DateTime::from(i.closed_at.unwrap());
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "confidential" => {
                    if i.confidential {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                "created_on" =>
                        if human {
                            r.push(Cell::new(HumanTime::from(i.created_at)))
                        } else {
                            let d: DateTime<Local> = DateTime::from(i.created_at);
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                "downvotes" => r.push(Cell::new(i.downvotes).set_alignment(CellAlignment::Right)),
                "due_date" =>
                    if i.due_date.is_some() {
                        r.push(Cell::new(i.due_date.unwrap()))
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    },
                "id" => r.push(Cell::new(i.iid).set_alignment(CellAlignment::Right)),
                "labels" => {
                    if !i.labels.is_empty() {
                        r.push(
                            Cell::new(
                                i.labels.join(",")
                            )
                        )
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "locked" => {
                    if i.discussion_locked.is_some() && i.discussion_locked.unwrap() {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                "mr" => r.push(Cell::new(i.merge_requests_count).set_alignment(CellAlignment::Right)),
                "state" => r.push(Cell::new(i.state.clone())),
                "subscribed" => {
                    if i.subscribed.is_some() && i.subscribed.unwrap() {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                "title" => r.push(Cell::new(i.title.clone())),
                "updated_on" => 
                        if human {
                            r.push(Cell::new(HumanTime::from(i.updated_at)))
                        } else {
                            let d: DateTime<Local> = DateTime::from(i.updated_at);
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                "upvotes" => r.push(Cell::new(i.upvotes).set_alignment(CellAlignment::Right)),
                "weight" =>
                    if i.weight.is_some() {
                        r.push(Cell::new(i.weight.unwrap()).set_alignment(CellAlignment::Right))
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Right))
                    },
                _ => unreachable!(""),
            }
        }

        if i.state == "opened" {
            r = r.iter().map(|f| f.clone().add_attribute(Attribute::Bold)).collect();
        } else {
            r = r.iter().map(|f| f.clone().add_attribute(Attribute::Dim)).collect();
        }

        if i.state == "opened" {
            r = r.iter().map(|f| f.clone().add_attribute(Attribute::Bold)).collect();
        } else if i.state == "closed" {
            r = r.iter().map(|f| f.clone().add_attribute(Attribute::Dim)).collect();
        } else {
            // r = r.iter().map(|f| f.clone().fg(Color::Yellow)).collect();
        }

        table.add_row(r);
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

            print_issues(
                issues,
                values_t_or_exit!(args, "fields", String),
                args.occurrences_of("no_headers")>0,
                args.occurrences_of("human_friendly")>0
                );

            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
