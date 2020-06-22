use crate::config;
use crate::subcommand;
use anyhow::Result;

pub struct MergeRequestCmd<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl subcommand::SubCommand for MergeRequestCmd<'_> {
    fn gen_clap_command(&self) -> clap::App {
        // TODO: figure out a way to get the borrow checker to work without `clone()`
        let c = self.clap_cmd.clone();
        c.about("Creates merge request")
            .visible_alias("mr")
            .arg(
                clap::Arg::with_name("description")
                    .short("d")
                    .long("description"),
            )
    }

    fn run(&self, config: config::Config, args: clap::ArgMatches) -> Result<()> {
        println!("Not implemented yet");
        trace!("Starting run()");
        trace!("Config: {:?}", config);
        trace!("Args: {:?}", args);
        Ok(())
    }
}
