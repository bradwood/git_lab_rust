use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use clap::value_t_or_exit;
use comfy_table::*;
use serde::Deserialize;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::converter::{
    mr_order_by_from_str, mr_scope_from_str, mr_state_from_str,
};
use crate::gitlab::{api, Client, MergeRequests, MergeRequestsBuilder, Query, SortOrder};

use crate::utils;

#[derive(Debug, Deserialize)]
pub struct MergeRequest {
    iid: u64,
    title: String,
    state: String,
    created_at: DateTime<Utc>,
}

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
            _ => unreachable!(),
        };
    }
    m.build()
        .map_err(|e| anyhow!("Could not construct merge requests query.\n {}", e))
}

fn print_mrs(mrs: Vec<MergeRequest>) {
    let mut table = Table::new();
    table
        .load_preset("                   ")
        .set_content_arrangement(ContentArrangement::Dynamic);

    for m in mrs {
        let create_date  = format!("{}", HumanTime::from(m.created_at));

        let id = if m.state == "opened" {
            Cell::new(m.iid).add_attribute(Attribute::Bold).fg(Color::Yellow)
        } else {
            Cell::new(m.iid).add_attribute(Attribute::Dim)
        };

        let title = if m.state == "opened" {
            Cell::new(m.title).add_attribute(Attribute::Bold)
        } else {
            Cell::new(m.title).add_attribute(Attribute::Dim)
        };

        table.add_row(vec![
            id,
            title,
            Cell::new("about ".to_string() + &create_date).add_attribute(Attribute::Dim),
        ]);
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

            print_mrs(mrs);

            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
