use crate::subcommand;

pub struct MergeRequest<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl subcommand::SubCommand for MergeRequest<'_> {
    fn gen_clap_command(&self) -> clap::App {
        // TODO: figure out a way to get the borrow checker to work without `clone()`
        let c = self.clap_cmd.clone();
        c.setting(clap::AppSettings::ColoredHelp)
            .about("Creates merge request")
            .visible_alias("mr")
            .arg(
                clap::Arg::with_name("description")
                    .short("d")
                    .long("description"),
            )
    }

    fn run(&self) {
        println!("I am the run() method on MergeRequest")
    }
}
