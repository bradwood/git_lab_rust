use anyhow::{anyhow, Context, Result};
use chrono::{Utc, DateTime, Local};
use chrono_humanize::HumanTime;
use clap::{value_t_or_exit, values_t_or_exit};
use comfy_table::*;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::converter::{
    mr_order_by_from_str, mr_scope_from_str, mr_state_from_str,
};
use crate::gitlab::{api, Client, MergeRequests, MergeRequestsBuilder, Query, SortOrder};
use crate::utils;
use crate::cmds::mr::MergeRequest;

pub fn generate_mrs_builder<'a>(
    args: &'a clap::ArgMatches,
    config: &'a config::Config,
    m: &'a mut MergeRequestsBuilder<'a>,
) -> Result<MergeRequests<'a>> {
    let project_id = utils::get_proj_from_arg_or_conf(&args, &config)?;
    m.project(project_id);

    for arg in &args.args {
        let (key, _) = arg;
        match *key {
            "state" if args.value_of("state").unwrap() != "all" => {
                m.state(mr_state_from_str(args.value_of("state").unwrap()).unwrap())
            }
            "state" if args.value_of("state").unwrap() == "all" => m,
            "scope" => m.scope(mr_scope_from_str(args.value_of("scope").unwrap()).unwrap()),
            "labels" => m.labels(args.values_of("labels").unwrap()),
            "unlabelled" => m.unlabeled(),
            "labelled" => m.with_any_label(),
            "author" => m.author(args.value_of("author").unwrap()),
            "approved_by" => m.approved_by_ids(
                utils::map_user_ids_from_names(
                    &config.members,
                    args.values_of("approved_by").unwrap()
                    )?
                .into_iter()
                ),
            "no_approvals" => m.no_approvals(),
            "any_approvals" => m.any_approvals(),
            "approvers" => m.approver_ids(
                utils::map_user_ids_from_names(
                    &config.members,
                    args.values_of("approvers").unwrap()
                    )?
                .into_iter()
                ),
            "no_approvers" => m.no_approvers(),
            "any_approvers" => m.any_approvers(),
            "assignee" => m.assignee_id(
                utils::map_user_ids_from_names(&config.members, args.values_of("assignee").unwrap())?[0]
                ),
            "assigned" => m.assigned(),
            "unassigned" => m.unassigned(),
            "filter" => m.search(args.value_of("filter").unwrap()),
            "created_after" => m.created_after(datefield!("created_after", args)),
            "created_before" => m.created_before(datefield!("created_before", args)),
            "updated_after" => m.updated_after(datefield!("updated_after", args)),
            "updated_before" => m.updated_before(datefield!("updated_before", args)),
            "wip" => m.wip(true),
            "order_by" => {
                m.order_by(mr_order_by_from_str(args.value_of("order_by").unwrap()).unwrap())
            }
            "descending" => m.sort(SortOrder::Descending),
            "ascending" => m.sort(SortOrder::Ascending),
            "max" => m,
            "fields" => m,
            "no_headers" => m,
            "human_friendly" => m,
            _ => unreachable!(),
        };
    }
    m.build()
        .map_err(|e| anyhow!("Could not construct merge requests query.\n {}", e))
}

fn print_mrs(mrs: Vec<MergeRequest>, fields: Vec<String>, no_headers: bool, human: bool) {
    let mut table = Table::new();

    table
        .load_preset("                   ")
        .set_content_arrangement(ContentArrangement::Dynamic);

    if !no_headers {
        table.add_row(fields.iter().map(|f| Cell::new(f.to_uppercase().replace("_"," ")).set_alignment(CellAlignment::Center)));
    }

    for m in mrs {
        let mut r: Vec<Cell> =Vec::new();

        for field in &fields {
            match field.as_str() {
                "assignees" => {
                    if m.assignees.is_some() {
                        r.push(
                            Cell::new(
                                m.assignees.clone()
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
                "author" => r.push(Cell::new(m.author["username"].as_str().unwrap())),
                "closed_by" => {
                    if m.closed_by.is_some() {
                        r.push(Cell::new(m.closed_by.clone().unwrap()["username"].as_str().unwrap()))
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "closed_on" => {
                    if m.closed_at.is_some() {
                        if human {
                            r.push(Cell::new(HumanTime::from(m.closed_at.unwrap())))
                        } else {
                            let d: DateTime<Local> = DateTime::from(m.closed_at.unwrap());
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "created_on" =>
                        if human {
                            r.push(Cell::new(HumanTime::from(m.created_at)))
                        } else {
                            let d: DateTime<Local> = DateTime::from(m.created_at);
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                "downvotes" => r.push(Cell::new(m.downvotes).set_alignment(CellAlignment::Right)),
                "has_conflicts" => {
                    if m.has_conflicts {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                "id" => r.push(Cell::new(m.iid).set_alignment(CellAlignment::Right)),
                "labels" => {
                    if !m.labels.is_empty() {
                        r.push(
                            Cell::new(
                                m.labels.join(",")
                            )
                        )
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "locked" => {
                    if m.discussion_locked.is_some() && m.discussion_locked.unwrap() {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                "merged_by" => {
                    if m.merged_by.is_some() {
                        r.push(Cell::new(m.merged_by.clone().unwrap()["username"].as_str().unwrap()))
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                "merged_on" => {
                    if m.merged_at.is_some() {
                        if human {
                            r.push(Cell::new(HumanTime::from(m.merged_at.unwrap())))
                        } else {
                            let d: DateTime<Local> = DateTime::from(m.merged_at.unwrap());
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                    } else {
                        r.push(Cell::new("-").set_alignment(CellAlignment::Center))
                    }
                },
                // "merge_status" => r.push(Cell::new(m.merge_status.clone())),
                "state" => r.push(Cell::new(m.state.clone())),
                "subscribed" => {
                    if m.subscribed.is_some() && m.subscribed.unwrap() {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                "title" => r.push(Cell::new(m.title.clone())),
                "source_branch" => r.push(Cell::new(m.source_branch.clone())),
                "target_branch" => r.push(Cell::new(m.target_branch.clone())),
                "updated_on" =>
                        if human {
                            r.push(Cell::new(HumanTime::from(m.updated_at)))
                        } else {
                            let d: DateTime<Local> = DateTime::from(m.updated_at);
                            r.push(Cell::new(d.format("%Y-%m-%d %H:%M:%S").to_string()))
                        }
                "upvotes" => r.push(Cell::new(m.upvotes).set_alignment(CellAlignment::Right)),
                "wip" => {
                    if m.work_in_progress {
                        r.push(Cell::new("y").set_alignment(CellAlignment::Center))
                    } else {
                        r.push(Cell::new("n").set_alignment(CellAlignment::Center))
                    }
                },
                _ => unreachable!(""),
            }
        }

        if m.state == "opened" {
            r = r.iter().map(|f| f.clone().add_attribute(Attribute::Bold)).collect();
        } else if m.state == "closed" {
            r = r.iter().map(|f| f.clone().add_attribute(Attribute::Dim)).collect();
        }
        // else {
        //     // r = r.iter().map(|f| f.clone().fg(Color::Yellow)).collect();
        // }

        table.add_row(r);
    }
    println!("{}", table);
}


pub fn list_mrs_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut i = MergeRequests::builder();
    let endpoint = generate_mrs_builder(&args, &config, &mut i)?;
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
            let mrs: Vec<MergeRequest> = api::paged(endpoint, api::Pagination::Limit(max as usize))
                .query(&gitlabclient)
                .context("Failed to query issues")?;


            print_mrs(
                mrs,
                values_t_or_exit!(args, "fields", String),
                args.occurrences_of("no_headers")>0,
                args.occurrences_of("human_friendly")>0
                );
            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
