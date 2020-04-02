// use clap::{App, Arg};

use crate::cmds::gen_subc;

pub struct Init<'a> {
    clap_cmd: clap::App<'a, 'a>,
}

impl gen_subc::GenSubCommand for Init<'_> {
    fn gen_clap_command(&self) -> clap::App {
        self.clap_cmd.about("Initialises GitLab server access").arg(
            clap::Arg::with_name("description")
                .short("d")
                .long("description"),
        )
    }
}

// impl gen_subc::GenSubCommand for clap::App<'_, '_> {
//     fn gen_subcommand(cmd: clap::App) -> clap::App<'_,'_> {
//         cmd.about("Initialises GitLab server access").arg(
//             clap::Arg::with_name("description")
//                 .short("d")
//                 .long("description")
//                 .to_owned(),
//         )
//     }
// }
