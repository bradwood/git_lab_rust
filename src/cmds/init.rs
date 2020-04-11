use crate::config;
use crate::config::GitConfigSaveableLevel::{Repo, User};
use crate::subcommand;
use anyhow::{Context, Result};
use dialoguer::{Input, PasswordInput};

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
            .setting(clap::AppSettings::ColoredHelp)
            .after_help("TODO: explain about env Vars. This is the after help.")
            .arg(
                clap::Arg::with_name("user")
                    .short("u")
                    .long("user")
                    .long_help(
"This swtich will cause the user-level rather than repo-level configuration to be updated. Note \
that this defaults to true when the command is invoked outside the directory hierarchy of a local \
git repo. If a local repo is found, its local repo-specific config will be updated unless this \
flag is passed.")
                    .help("Set credentials at user scope"),
            )
    }


    fn run(&self, mut config: config::Config, args: clap::ArgMatches) -> Result<()> {
        trace!("Starting run()");
        trace!("Config: {:?}", config);
        trace!("Args: {:?}", args);
        trace!("--user : {:?}", args.is_present("user"));


        // for each entry in config object, prompt for the item
        // setting defaults from the config object, if present
        config.host = Input::<String>::new()
            .with_prompt("GitLab host")
            .default(config.host.unwrap_or_else(|| "None".to_string()))
            .interact().ok();
        config.token = PasswordInput::new()
            .with_prompt("GitLab personal access token")
            .interact().ok();
        config.tls = Input::<bool>::new()
            .with_prompt("TLS enabled")
            .default(config.tls.unwrap_or(true))
            .interact().ok();

        // default to local config unless --user is passed
        // default to --user if git_repo cannot be found.
        let write_user = config.repo_path.is_none() || args.is_present("user");
        trace!("writing to user config? : {:?}", write_user);

        config.save(Repo).context("Could not save to git config")?;

        // FIXME

        Ok(())
    }
}
