use anyhow::{anyhow, Context, Result};
use chrono_humanize::HumanTime;
use colored::*;
use lazy_static::*;
use regex::Regex;
use termimad::*;
use textwrap::{fill, indent, termwidth};

use crate::cmds::mr::{generate_basic_mr_builder, MergeRequest};
use crate::config;
use crate::config::OutputFormat;
use crate::gitlab::MergeRequest as GLMergeRequest;
use crate::gitlab::{api, Client, Query};

fn print_mr(m: MergeRequest) {
    let mut skin = MadSkin::default();
    skin.headers[0].align = Alignment::Left;
    skin.code_block.align = Alignment::Center;
    let c_date = format!("{}", HumanTime::from(m.created_at));
    let u_date = format!("{}", HumanTime::from(m.updated_at));
    let up = format!("{}", "u".dimmed());
    let down = format!("{}", "d".dimmed());
    let merging_into = format!("{}", ">".dimmed());
    let dot = format!("{}", "•".dimmed());
    let comments = format!("{}", "comments".dimmed());
    let assignee_str = format!("{}", "assigned".italic().blue().bold());
    let updated = format!("{}", "updated".dimmed());
    let m_status = match m.merge_status.as_str() {
        "can_be_merged" if m.state == "opened" => "can be merged".to_string().italic().bold(),
        "cannot_be_merged" if m.state == "opened" => "cannot be merged".to_string().italic().bold(),
        _ => "".to_string().bold(),
    };

    println!("{} ", m.title.bold());

    // sub title info line
    println!(
        "{}   {} {} {} {} {} {} {} {} {}{} {} {}{} {} {}",
        m.state.italic().blue().bold(),
        dot,
        m.author["name"].as_str().unwrap().dimmed(),
        m.state.dimmed(),
        c_date.dimmed(),
        dot,
        updated,
        u_date.dimmed(),
        dot,
        m.upvotes.to_string().dimmed(),
        up,
        dot,
        m.downvotes.to_string().dimmed(),
        down,
        dot,
        m_status
    );

    println!(
        "         {} {} {} {} {}",
        dot,
        m.references["full"].as_str().unwrap().dimmed(),
        dot,
        m.user_notes_count.to_string().dimmed(),
        comments,
    );

    println!(
        "         {} {} {} {}",
        dot,
        m.source_branch.italic().bold(),
        merging_into,
        m.target_branch.italic(),
    );

    let assignee_names = m
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

    lazy_static! {
        static ref WHITESPACE_RE: Regex = Regex::new(r"\s").unwrap();
    }
    const NBSP: char = '\u{a0}';

    if !m.labels.is_empty() {
        let label_str = m
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

    // print the entire description
    if m.description.is_some() {
        let desc_text = m.description.unwrap();
        let mut area = Area::full_screen();
        area.pad(6, 0);
        let md = skin.area_text(desc_text.as_str(), &area).to_string();
        //TODO: should this also configure the local to track this new remote branch??? 

        let indent_md = indent(&md, "    ");
        println!("{}", &indent_md);

    }
    println!(
        "{} {}",
        "View this merge request on GitLab:".italic().dimmed(),
        m.web_url.italic().dimmed()
    );
}

pub fn show_mr_cmd(
    args: clap::ArgMatches,
  config: config::Config,
    gitlabclient: Client,
) -> Result<()> {
    let mut i = GLMergeRequest::builder();
    let endpoint = generate_basic_mr_builder(&args,"id", &config, &mut i)?;

 debug!("args: {:#?}", args);
    debug!("endpoint: {:#?}", endpoint);

    match config.format {
        Some(OutputFormat::JSON) => {
            let raw_json = api::raw(endpoint)
                .query(&gitlabclient)
                .context("Failed to find merge request")?;

            println!("{}", String::from_utf8(raw_json).unwrap());
            Ok(())
        }

        Some(OutputFormat::Text) => {
            let mr: MergeRequest = endpoint
                .query(&gitlabclient)
                .context("Failed to find merge request")?;

            print_mr(mr);
            Ok(())
        }
        _ => Err(anyhow!("Bad output format in config")),
    }
}
