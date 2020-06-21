use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use clap::value_t_or_exit;
use humantime::parse_duration;

use crate::cmds::issue::Issue;
use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::converter::{
    issue_order_by_from_str, issue_scope_from_str, issue_state_from_str,
};
use crate::gitlab::{api, Client, IssueWeight, Issues, IssuesBuilder, Query, SortOrder};
use crate::utils;

macro_rules! datefield {
    ($s:expr, $a:expr) => {
        Utc::now() - Duration::from_std(parse_duration($a.value_of($s).unwrap()).unwrap()).unwrap()
    };
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
            _ => unreachable!(),
        };
    }
    i.build()
        .map_err(|e| anyhow!("Could not construct issues query from server.\n {}",e))
}

//TODO: Handle pagination/records returned
pub fn list_issues_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut i = Issues::builder();
    let endpoint = generate_issues_builder(&args, &config, &mut i)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json = api::raw(endpoint).query(&gitlabclient).context(
                "Failed to query issues",
            )?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        }

        Some(OutputFormat::Text) => {
            let issues: Vec<Issue> = endpoint.query(&gitlabclient).context(
                "Failed to query issues",
            )?;

            println!("Issue id: {}", issues[0].id);
            println!("Issue URL: {}", issues[0].web_url);
            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
