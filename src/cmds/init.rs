use anyhow::{Context, Result};
use dialoguer::{Input, PasswordInput, Select};

use crate::config;
use crate::config::GitConfigSaveableLevel::{Repo, User};
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
        c.about("Initialises server credentials interactively")
            .setting(clap::AppSettings::ColoredHelp)
            .after_help("This command initialises the various configuration data needed to talk to \
a GitLab server API. It does this interactively, by prompting the user for the data required. The \
configuration data is saved using the standard git-config(1) mechanics and conventions (including \
precedence), at the user- or repo-specific level (see `--user`). As this is just standard git \
config, one can simply edit the appropriate git config file directly or invoke git-config(1) as \
illustrated in the examples below:

    git config --local --add gitlab.host my.gitlab.host.com
    git config --local --add gitlab.token PERSONAL_ACCESS_TOKEN
    git config --global --add gitlab.tls true
    git config --global --add gitlab.format json

Initialisation via `git lab init` is not mandatory. Users preferring to set configuration \
parameters by environment variables can do so. The variables that can be set are shown below. Note \
that setting these will override the data set in any git config file.

    GITLABCLI_HOST
    GITLABCLI_TOKEN
    GITLABCLI_TLS
    GITLABCLI_FORMAT
")

            .arg(
                clap::Arg::with_name("user")
                    .short("u")
                    .long("user")
                    .long_help(
"This swtich will cause the user-level (either $HOME/.gitconfig or $XDG_CONFIG_HOME/git/config) \
rather than repo-level ($GITDIR/.git/config) configuration to be updated. Note that this defaults \
to true when the command is invoked outside the directory hierarchy of a local git repo. If a local \
repo is found, its local repo-specific config will be updated unless this flag is passed. If you \
wish to manage your configuration across a combination of git config files (e.g., system, global \
and local) then you must directly edit the relevant files or invoke git-config(1) directly.")
                    .help("Set credentials at user scope"),
            )
    }

    fn run(&self, mut config: config::Config, args: clap::ArgMatches) -> Result<()> {
        trace!("Starting run()");
        trace!("Config: {:?}", config);
        trace!("Args: {:?}", args);
        trace!("--user : {:?}", args.is_present("user"));

        // Get config from user
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

        let format_options = &["Text", "JSON"];
        let format_choice = Select::new()
            .with_prompt("Output format")
            .default(
                format_options
                .iter()
                .position(
                    |&x| x == config.format.as_ref().unwrap_or(&config::OutputFormat::Text).to_string()
                )
                .unwrap()
            )
            .items(&format_options[..])
            .interact().unwrap();

        config.format = format_options[format_choice].parse().ok();

        // Write to appropriate config file
        if config.repo_path.is_none() || args.is_present("user") {
            config.save(User).with_context(|| format!("Could not save to git config: {:?}", User))?;
            println!("Updated user {:?} config", config.user_config_type.unwrap());
        } else {
            config.save(Repo).with_context(|| format!("Could not save to git config: {:?}", Repo))?;
            println!("Updated repo config {:?}", config.repo_path.unwrap());
        }
        Ok(())
    }
}
