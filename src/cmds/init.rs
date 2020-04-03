use clap::AppSettings;

use crate::gen_subc;

pub struct Init<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl gen_subc::GenSubCommand for Init<'_> {
    fn gen_clap_command(&self) -> clap::App {
        // TODO: figure out a way to get the borrow checker to work without `clone()`
        let c = self.clap_cmd.clone();
        c.setting(AppSettings::ColoredHelp)
            .about("Initialises GitLab server access")
            .arg(
                clap::Arg::with_name("description")
                    .short("d")
                    .long("description"),
            )
    }
}
