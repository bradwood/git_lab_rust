// use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};
// mod config;

mod cmds {
    pub mod gen_subc;
    pub mod init;
    pub mod merge_request;
}

mod args;

// use config::Config;

fn main() {
    let command = args::get_args();
    // let config = Config::defaults();

    // match command {
    //     Command::Add(s) => {
    //         println!("Add: {}", s);
    //         add(config.file, s)
    //     }
    //     Command::Increase(x) => println!("Increase: {}", x),
    //     Command::Decrease(x) => println!("Decrease: {}", x),
    //     Command::Purge => println!("Purge"),
    //     Command::Stats => println!("Stats"),
    //     Command::Complete => println!("Complete"),
    //     Command::Directory(v) => println!("Directory: {:?}", v),
    //     Command::Error(e) => {
    //         println!("Error: {}", e);
    //         std::process::exit(1)
    //     }
    // }
}
