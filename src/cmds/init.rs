use crate::config;
use crate::subcommand;
/// This implements the `init` command. It initialises the GitLab-specific config data needed to
/// communicate with the server. See [`config`] for more details.
///
/// [`config`]: ../../config/struct.Config.html
pub struct Init<'a> {
    pub clap_cmd: clap::App<'a, 'a>,
}

impl subcommand::SubCommand for Init<'_> {
    fn gen_clap_command(&self) -> clap::App {
        // TODO: figure out a way to get the borrow checker to work without `clone()`
        let c = self.clap_cmd.clone();
        c.about("Initialises server credentials")
            .arg(
                clap::Arg::with_name("host")
                    .short("h")
                    .long("host")
                    .takes_value(true)
                    .help("Hostname of GitLab server"),
            )
            .arg(
                clap::Arg::with_name("token")
                    .short("t")
                    .long("token")
                    .takes_value(true)
                    .help("Personal access token"),
            )
            .arg(
                clap::Arg::with_name("no-tls")
                    .short("n")
                    .long("no-tls")
                    .visible_alias("no-ssl")
                    .help("Don't use TLS (ssl) for server communication"),
            )
    }

    fn run(&self, config: config::Config, args: clap::ArgMatches) {
        trace!("Starting run()");
        trace!("Config: {:?}", config);
        trace!("Args: {:?}", args);
        // trace!("Verbosity: {}", verbosity);
    }
}
