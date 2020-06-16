use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;
use chrono_humanize::HumanTime;
use colored::*;
use serde::Deserialize;
use serde_json::{Map, Value};
use termimad::*;
use textwrap::indent;

use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::{api, Client, Query};
use crate::gitlab::Issue as GLIssue;
use crate::cmds::issue::open;

#[derive(Debug, Deserialize)]
struct Issue {
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: Option<String>,
    state: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    closed_by: Option<Map<String, Value>>,
    labels: Vec<String>,
    milestone: Option<String>,
    author: Map<String, Value>,
    assignees: Option<Vec<Map<String, Value>>>,
    user_notes_count: u64,
    merge_requests_count: u64,
    upvotes: u64,
    downvotes: u64,
    due_date: Option<NaiveDate>,
    confidential: bool,
    discussion_locked: Option<bool>,
    web_url: String,
    task_completion_status: Option<Map<String, Value>>,
    weight: Option<u64>,
    has_tasks: Option<bool>,
    task_status: Option<String>,
    references: Map<String, Value>,
    subscribed: Option<bool>,
}

fn print_issue(i: Issue) {
    let mut skin = MadSkin::default();
    skin.headers[0].align = Alignment::Left;
    skin.code_block.align = Alignment::Center;
    let c_date = format!("{}", HumanTime::from(i.created_at));
    let u_date = format!("{}", HumanTime::from(i.updated_at));
    let up = format!("{}", "🠕".dimmed());
    let down = format!("{}", "🠗".dimmed());
    let merge = format!("{}", "∧".dimmed());
    let weight = format!("{}", "ω".dimmed());
    let dot = format!("{}", "•".dimmed());
    let comments = format!("{}", "comments".dimmed());
    let updated = format!("{}", "updated".dimmed());
    let due = format!("{}", "due".dimmed());

    // title
    println!("{} ",
        i.title.bold(),
        );

    // sub title info line
    print!("{} {} {} {} {} {} {} {} {} {}{} {} {}{} {} {}{} ",
        i.state.italic().blue().bold(),
        dot,
        i.author["name"].as_str().unwrap().dimmed(),
        i.state.dimmed(),
        c_date.dimmed(),
        dot,
        updated,
        u_date.dimmed(),
        dot,
        i.upvotes.to_string().dimmed(),
        up,
        dot,
        i.downvotes.to_string().dimmed(),
        down,
        dot,
        i.merge_requests_count.to_string().dimmed(),
        merge,
        );

    if i.weight.is_some() {
        println!("{} {}{}",
            dot,
            i.weight.unwrap().to_string().dimmed(),
            weight,
            )
    } else {
        println!();
    }

    // 2nd sub title info line
    print!("       {} {} {} {} {}",
        dot,
        i.references["full"].as_str().unwrap().dimmed(),
        dot,
        i.user_notes_count.to_string().dimmed(),
        comments,
        );

    // print tasks info if issue has tasks
    if i.has_tasks.is_some() && i.has_tasks.unwrap() {
        print!(" {} {}",
            dot,
            i.task_status.unwrap().dimmed(),
            );
    }

    // print due date if present
    if i.due_date.is_some() {
        let d  = format!("{}", HumanTime::from(Utc.from_utc_date(&i.due_date.unwrap()).and_hms(0,0,0)));
        print!(" {} {} {}",
            dot,
            due,
            d.dimmed(),
            );
    }
    println!();

    //print labels
    if !i.labels.is_empty() {
    print!("labels • ");
        for l in i.labels {
            print!("{} ", l);
        }
    }

    println!("\n");

    // print the entire issue description
    if i.description.is_some() {
        let desc_text =  i.description.unwrap();
        let mut area = Area::full_screen();
        area.pad(6,0);
        let md = skin.area_text(desc_text.as_str(), &area).to_string();

        let indent_md = indent(&md, "    "); 
        println!("{}", &indent_md);

        println!("{} {}",
            "View this issue on GitLab:".italic().dimmed(),
            i.web_url.italic().dimmed()
        );
    }

}

pub fn show_issue_cmd(args: clap::ArgMatches, config: config::Config, gitlabclient: Client) -> Result<()> {
    let mut p = GLIssue::builder();
    let endpoint = open::generate_issue_builder(&args, &config, &mut p)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json  = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to find issue")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        },

        Some(OutputFormat::Text) => {
            let issue: Issue = endpoint
                .query(&gitlabclient)
                .context("Failed to find issue")?;

            print_issue(issue);
            Ok(())
        },
        _ => Err(anyhow!("Bad output format in config")),
    }
}

