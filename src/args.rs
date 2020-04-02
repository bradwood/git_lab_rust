use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

use crate::cmds::gen_subc;
use crate::cmds::init;

#[derive(Debug)]
pub enum Command {
    Add(String),
    Increase(f64),
    Decrease(f64),
    Purge,
    Stats,
    Complete,
    Directory(Vec<String>),
    Error(String),
}

pub fn get_args() -> Command {

    let cli_commands = gen_subc::ClapCommands {
        commands: vec![Box::new(
                      init::Init {clap_cmd: SubCommand::with_name("init")},
                      )],
    };
    // TODO: add a trait for the gen_subcommand method
    // for item in subcs, do:
    // import module of that name
    // call gen_subcommand with string passed and returning Command
    // create command_vector
    // pass into get_args()

    let matches = App::new("git-lab")
        .setting(AppSettings::ColoredHelp)
        // .setting(AppSettings::UnifiedHelpMessage)
        // .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!())
        .about("GitLab cli")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Set verbosity level")
                .multiple(true),
        )
        .subcommands(cli_commands.generate())
        .get_matches();

    println!("Matches = {:#?}", matches);
    Command::Purge

    // if let Some(o) = matches.value_of("add") {
    //     Command::Add(String::from(o))
    // } else if matches.is_present("increase") {
    //     match matches.value_of("increase") {
    //         Some(x) => match x.parse::<f64>() {
    //             Err(s) => Command::Error(s.to_string()),
    //             Ok(f) if f > 0.0 => Command::Increase(f),
    //             Ok(_) => Command::Error("non-positive parameter passed".to_string()),
    //         },
    //         None => Command::Increase(10.0),
    //     }
    // } else if matches.is_present("decrease") {
    //     match matches.value_of("decrease") {
    //         Some(x) => match x.parse::<f64>() {
    //             Err(s) => Command::Error(s.to_string()),
    //             Ok(f) if f > 0.0 => Command::Decrease(f),
    //             Ok(_) => Command::Error("non-positive parameter passed".to_string()),
    //         },
    //         None => Command::Decrease(15.0),
    //     }
    // } else if matches.is_present("purge") {
    //     Command::Purge
    // } else if matches.is_present("stats") {
    //     Command::Stats
    // } else if matches.is_present("complete") {
    //     Command::Complete
    // } else {
    //     Command::Directory(values_t!(matches, "DIRECTORY", String).unwrap())
    // }
}
