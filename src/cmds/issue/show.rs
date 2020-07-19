use anyhow::{anyhow, Context, Result};
use chrono::offset::TimeZone;
use chrono::Utc;
use chrono_humanize::HumanTime;
use colored::*;
use lazy_static::*;
use regex::Regex;
use termimad::*;
use textwrap::{fill, indent, termwidth};

use crate::cmds::issue::{generate_basic_issue_builder, Issue};
use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::Issue as GLIssue;
use crate::gitlab::{api, Client, Query};

fn print_issue(i: Issue) {
    let mut skin = MadSkin::default();
    skin.headers[0].align = Alignment::Left;
    skin.code_block.align = Alignment::Center;
    let c_date = format!("{}", HumanTime::from(i.created_at));
    let u_date = format!("{}", HumanTime::from(i.updated_at));
    let up = format!("{}", "u".dimmed());
    let down = format!("{}", "d".dimmed());
    let merge = format!("{}", "m".dimmed());
    let weight = format!("{}", "w".dimmed());
    let dot = format!("{}", "•".dimmed());
    let comments = format!("{}", "comments".dimmed());
    let assignee_str = format!("{}", "assigned".italic().blue().bold());
    let updated = format!("{}", "updated".dimmed());
    let due = format!("{}", "due".dimmed());
    let confidential = format!("{}", "c".dimmed());

    // title
    println!("{} ", i.title.bold());

    // sub title info line
    print!(
        "{}   {} {} {} {} {} {} {} {} {}{} {} {}{} {} {}{} ",
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
        print!(
            "{} {}{}",
            dot,
            i.weight.unwrap().to_string().dimmed(),
            weight,
        )
    }

    if i.confidential {
        print!("{} {}", dot, confidential,)
    } else {
        println!();
    }

    // 2nd sub title info line
    print!(
        "         {} {} {} {} {}",
        dot,
        i.references["full"].as_str().unwrap().dimmed(),
        dot,
        i.user_notes_count.to_string().dimmed(),
        comments,
    );

    // print tasks info if issue has tasks
    if i.has_tasks.is_some() && i.has_tasks.unwrap() {
        print!(" {} {}", dot, i.task_status.unwrap().dimmed(),);
    }

    // print due date if present
    if i.due_date.is_some() {
        let d = format!(
            "{}",
            HumanTime::from(Utc.from_utc_date(&i.due_date.unwrap()).and_hms(0, 0, 0))
        );
        print!(" {} {} {}", dot, due, d.dimmed(),);
    }
    println!();

    // print assignees

    let assignee_names = i
        .assignees
        .unwrap()
        .iter()
        .map(|e| e["username"].as_str().unwrap().to_string())
        .collect::<Vec<String>>();

    if !assignee_names.is_empty() {

        println!(
            "{} {} {}",
            assignee_str,
            dot,
            assignee_names.join(&format!(" {} ", dot)).dimmed(),
            );
    }

    // print labels -- this is bit tricky, as we want to linewrap the labels, but not mid-label even if
    // it has a space in it. We tackle this by:
    // - substititing spaces _in_ the label with NBSPs
    // - we then generate a _single_ string of labels with spaces in between
    // - we then textwrap the result.
    lazy_static! {
        static ref WHITESPACE_RE: Regex = Regex::new(r"\s").unwrap();
    }
    const NBSP: char = '\u{a0}';

    if !i.labels.is_empty() {
        // print!("labels • ");

        let label_str = i
            .labels
            .iter()
            .map(|x| {
                WHITESPACE_RE
                    .replace_all(&x, NBSP.to_string().as_str())
                    .to_string()
            })
            .collect::<Vec<String>>()
            .join(&format!(" {} ", dot));

        print!(
            "{}",
            indent(&fill(&label_str, termwidth() - 14), "           ").italic()
        );
    }

    println!();

    // print the entire issue description
    if i.description.is_some() {
        let desc_text = i.description.unwrap();
        let mut area = Area::full_screen();
        area.pad(6, 0);
        let md = skin.area_text(desc_text.as_str(), &area).to_string();

        let indent_md = indent(&md, "    ");
        println!("{}", &indent_md);

    }
    println!(
        "{} {}",
        "View this issue on GitLab:".italic().dimmed(),
        i.web_url.italic().dimmed()
    );
}

pub fn show_issue_cmd(
    args: clap::ArgMatches,
    config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut i = GLIssue::builder();
    let endpoint = generate_basic_issue_builder(&args,"id", &config, &mut i)?;

    debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to find issue")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        }

        Some(OutputFormat::Text) => {
            let issue: Issue = endpoint
                .query(&gitlabclient)
                .context("Failed to find issue")?;

            print_issue(issue);
            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
