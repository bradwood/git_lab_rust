use std::env;
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use git2::Config as GitConfig;
use git2::ConfigLevel::{Global, Local, System, XDG};
use git2::ConfigLevel;
use git2::Repository;

use crate::utils::find_git_root;

/// This enum specifies the two ways in which git config can be saved, either to the User's config
/// (dotfile) or to the Repo's.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum GitConfigSaveableLevel {
    Repo,
    User,
}

/// This enum specifies the two types of User-level git config: the XDG form, and the local user
/// dotfile form (confusingly, known as `Global` in git parlance).
#[derive(Debug)]
#[derive(PartialEq)]
pub enum UserGitConfigLevel {
    XDG,
    Global,
}

/// This enum specifies the two different output formats supported
#[derive(Debug)]
#[derive(PartialEq)]
pub enum OutputFormat {
    Text,
    JSON,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "JSON" => Ok(OutputFormat::JSON),
            "TEXT" => Ok(OutputFormat::Text),
            _ => Err(anyhow!("Bad output format: {}", s)),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// This struct holds the config data required to talk to a GitLab server as well as other
/// configuration data, including the path to the local repo (if any).
///
/// It exploits the standard git-config(1) mechanics and thus can read this data from:
///  * `/etc/gitconfig` --- the __system__ config
///  * `$XDG_CONFIG_HOME/git/config` --- the __XDG__ config
///  * `$HOME/.gitconfig` --- the __global__ config
///  * `$GIT_DIR/.git/config` --- the repo-specific or __local__ config
///
/// Override priority increases from top to bottom.
#[derive(Debug)]
pub struct Config {
    pub token: Option<String>,
    pub host: Option<String>,
    pub tls: Option<bool>,
    pub format: Option<OutputFormat>,
    pub repo_path: Option<PathBuf>, //convenience param, not saved with ::save()
    pub user_config_type: Option<UserGitConfigLevel>, //convenience param, not saved with ::save()
    pub projectid: Option<u64>, //set with project attach command
}

/// Open System, XDG and Global multi-level config or return empty config.
fn maybe_open_multilevel_config() -> GitConfig {
    match GitConfig::open_default() {
        Ok(mlc) => {
            trace!("Opened multi-level default config");
            mlc
        }
        Err(_) => {
            warn!("Didn't find default git config, ignoring");
            GitConfig::new().unwrap()
        }
    }
}

/// Return the path to the local git repo if found.
fn maybe_get_local_repo() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    find_git_root(&cwd)
}

/// Open local ($REPODIR/.git/config) or return empty config.
fn maybe_open_local_config() -> GitConfig {
    // See https://stackoverflow.com/q/61119366/743861
    (|| {
        let git_path = maybe_get_local_repo()?;
        trace!("found local or upstream git repo");
        let repo = Repository::open(&git_path).ok()?;
        trace!("opened git repo");
        let config = repo.config().ok()?;
        trace!("opened git repo's local config");
        config.open_level(Local).ok()
    })().unwrap_or_else(|| GitConfig::new().unwrap())
}

/// Update this app's Config object from a git single-level config object
fn update_config_from_git(config: &mut Config, git_config: &GitConfig) {
    for entry in &git_config.entries(Some("gitlab")).unwrap() {
        let entry = entry.unwrap();
        match entry.name().unwrap() {
            "gitlab.token" => config.token = Some(entry.value().unwrap().to_string()),
            "gitlab.host" => config.host = Some(entry.value().unwrap().to_string()),
            "gitlab.tls" => config.tls = Some(
                entry.value().unwrap().to_uppercase() == "TRUE" ||
                entry.value().unwrap().to_uppercase() == "YES" ||
                entry.value().unwrap().to_uppercase() == "ON" ||
                entry.value().unwrap().to_uppercase() == "1"
                ),
            "gitlab.format" => config.format = entry.value().unwrap().to_string().parse::<OutputFormat>().ok(),
            "gitlab.projectid" => config.projectid = Some(entry.value().unwrap().parse::<u64>().unwrap()),
            _ => (),
        };
        trace!(
            "{:?} : {} <= {}",
            config,
            entry.name().unwrap(),
            entry.value().unwrap()
        );
    }
}

/// Update this app's Config object from environment variables if found
fn update_config_from_env<V>(config: &mut Config, vars: V)
where
    V: Iterator<Item = (String, String)> // use a trait bound to aid testing

{
    let gitlab_vars = vars.filter(|(k, _)| k.starts_with("GITLABCLI_"));
    for (key, value) in gitlab_vars {
        if key == "GITLABCLI_TOKEN" { config.token = Some(value); continue };
        if key == "GITLABCLI_HOST" { config.host = Some(value); continue };
        if key == "GITLABCLI_TLS" {
            config.tls = Some(
                value.to_uppercase() == "TRUE" ||
                value.to_uppercase() == "YES" ||
                value.to_uppercase() == "ON" ||
                value.to_uppercase() == "1"
                );
            continue
        };
        if key == "GITLABCLI_FORMAT" { config.format = value.parse::<OutputFormat>().ok(); continue };
        if key == "GITLABCLI_PROJECTID" { config.projectid = value.parse::<u64>().ok(); continue };
    }
}

/// Get a specific single level of git config from a multi-level config
fn get_level_config(multi_level: &GitConfig, level: ConfigLevel) -> GitConfig {
    match multi_level.open_level(level) {
        Ok(c) => {
            trace!("Opened config at level {:?}", level);
            c
        }
        Err(_) => {
            trace!( "Didn't find config at level {:?}, proceeding without", level);
            GitConfig::new().unwrap()
        }
    }
}

/// Return which type of user config is being used. Is it Global ($HOME/.gitconfig)
/// or XDG ($HOME/.config/git/config)? If none can be found, it will force Global.
/// If both are found return XDG
fn get_user_config_type() -> Option<UserGitConfigLevel> {

    if GitConfig::find_xdg().is_ok() {
        Some(UserGitConfigLevel::XDG)
    } else {
        Some(UserGitConfigLevel::Global)
    }
}

/// Write config data to a git config,
fn write_config(save_config: &mut GitConfig, config: &Config) -> Result<()> {

    if config.host.is_some()
        && ( env::var("GITLABCLI_HOST").is_err()
            || &env::var("GITLABCLI_HOST").unwrap() != config.host.as_ref().unwrap()
           )
    {
        save_config.set_str("gitlab.host", config.host.as_ref().unwrap())
            .context("Failed to save gitlab.host to git config.")?;
    }

    if config.token.is_some()
        && ( env::var("GITLABCLI_TOKEN").is_err()
            || &env::var("GITLABCLI_TOKEN").unwrap() != config.token.as_ref().unwrap()
           )
    {
        save_config.set_str("gitlab.token", config.token.as_ref().unwrap())
            .context("Failed to save gitlab.token to git config.")?;
    }

    // no environment checking for booleans, probably should be done at some point
    if config.tls.is_some() {
        save_config.set_bool("gitlab.tls", config.tls.unwrap())
            .context("Failed to save gitlab.tls to git config.")?;
    }

    if config.format.is_some()
        && ( env::var("GITLABCLI_FORMAT").is_err()
            || env::var("GITLABCLI_FORMAT").unwrap().to_lowercase() != config.format.as_ref().unwrap().to_string().to_lowercase()
           )
    {
        save_config.set_str("gitlab.format", config.format.as_ref().unwrap().to_string().to_lowercase().as_str())
            .context("Failed to save gitlab.format to git config.")?;
    }

    if config.projectid.is_some()
        && ( env::var("GITLABCLI_PROJECTID").is_err()
            || env::var("GITLABCLI_PROJECTID").unwrap() != config.projectid.as_ref().unwrap().to_string()
           )
    {
        save_config.set_i64("gitlab.projectid", i64::try_from(config.projectid.unwrap()).unwrap())
            .context("Failed to save gitlab.projectid to git config.")?;
    }

    Ok(())
}

impl Config {

    /// Create an empty config object.
    fn new() -> Config {
        Config {
            token: None,
            host: None,
            tls: None,
            format: None,
            projectid: None,
            repo_path: None,
            user_config_type: None,
        }
    }

    /// Reads the configs from the various GitLab sections in the various git config files and
    /// loads them into the Config struct.
    pub fn defaults() -> Config {
        trace!( "Creating empty Config object");
        let mut config = Self::new();

        trace!( "Get a local repo path if one is there");
        config.repo_path = maybe_get_local_repo();

        trace!( "Read multi-level git config (which excludes repo's config)");
        let default_config = maybe_open_multilevel_config();

        config.user_config_type = get_user_config_type();
        trace!( "User config file: {:?}", config.user_config_type.as_ref().unwrap());

        trace!( "Load config object data from System, XDG or Global git configs");
        static LEVELS: [ConfigLevel; 3] = [System, XDG, Global];
        #[allow(clippy::suspicious_map)] //using count() below to force iterator consumption
        LEVELS.iter()
            .map(|l|
                update_config_from_git(&mut config,
                    &get_level_config(&default_config, *l)
                    )
                )
            .count();

        trace!( "Open local repo-specific config if one was found");
        let local = maybe_open_local_config();

        trace!( "Override any previously set config data using Local config, if it was found");
        update_config_from_git(&mut config, &local);

        trace!( "Override any previously set config data using enivronment variables, if found");
        update_config_from_env(&mut config, env::vars());

        trace!( "Return config");
        config
    }

    /// Saves the config to the appropriate config file. NOTE it will apply XDG instead of Global
    /// if config.user_config_type is set to XDG, and vice versa.
    pub fn save(&self, level:GitConfigSaveableLevel) -> Result<()> {
        match level {
            GitConfigSaveableLevel::Repo => {
                let mut save_config = maybe_open_local_config();
                self.repo_path.as_ref().ok_or_else(|| anyhow!("Cannot save to local git repo config if it can't be found."))?;
                write_config(&mut save_config, self)?;
            },
            GitConfigSaveableLevel::User => {
                match self.user_config_type.as_ref().unwrap() {
                    UserGitConfigLevel::Global => {
                        let mut save_config = GitConfig::open(&GitConfig::find_global().unwrap()).unwrap();
                        write_config(&mut save_config, self)?;
                    },
                    UserGitConfigLevel::XDG => {
                        let mut save_config = GitConfig::open(&GitConfig::find_xdg().unwrap()).unwrap();
                        write_config(&mut save_config, self)?;
                    },
                }
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod config_unit_tests {
    use assert_fs::prelude::*;
    use lazy_static::*;
    use rstest::*;
    use std::path::Path;
    use std::sync::Once;

    use super::*;

    // TODO figure out a way of importing this from tests/common/mod.rs

    // This is a single, static TempDir used to run all tests in.
    lazy_static! {
        static ref HOME: assert_fs::TempDir = assert_fs::TempDir::new().unwrap().into_persistent();
    }
    // This is an initialisation function that runs once, and only once, for all tests. This is
    // needed due to the way git2-rs works, in that when the underlying gitlib2 C library is
    // initialised, it caches `sysdir` only once, on initialisation, and then disregards any
    // subsequent changes to env vars like $HOME and $XDG_CONFIG_HOME. This means that if we try to
    // change these vars during the test run, those changes are ignored. As a result, the env vars
    // are set once, and do not change, and we need to run every test within the same $HOME and
    // $XDG_CONFIG_HOME and each test needs to ensure this directory tree is set up for that test's
    // needs.
    static INIT: Once = Once::new();
    fn initialise() {
        INIT.call_once(|| {
            env::set_var("HOME", HOME.path());
            std::fs::write(HOME.child(".gitconfig").path(),"").unwrap();

            env::set_var("XDG_CONFIG_HOME", HOME.child(".config").path());
            std::fs::create_dir_all(HOME.child(".config/git").path()).unwrap();
            std::fs::write(HOME.child(".config/git/config").path(),"").unwrap();

            let repo_path = HOME.child("repo");
            Repository::init(repo_path.path()).unwrap();
            cd_home();
        });
    }

    // -- convenience functions --

    fn cd_home() {
        env::set_current_dir(
            Path::new(
                &HOME.path()
            )
        ).unwrap();
    }

    fn cd_repo() {
        env::set_current_dir(
            Path::new(
                &HOME.path()
            ).join("repo")
        ).unwrap();
    }

    fn reset_repo() {
        if std::path::Path::is_dir(HOME.child("repo").path()) {
            std::fs::remove_dir_all(HOME.child("repo").path()).unwrap();
        }
        let repo_path = HOME.child("repo");
        Repository::init(repo_path.path()).unwrap();
    }

    fn reset_global_config() {
        std::fs::write(HOME.child(".gitconfig").path(),"").unwrap();
    }

    fn reset_xdg_config() {
        std::fs::write(HOME.child(".config/git/config").path(),"").unwrap();
    }

    // -- maybe_get_local_repo --

    // #[test]
    // fn test_get_local_repo_no_cwd() {
    //     initialise();
    //     let temp = assert_fs::TempDir::new().unwrap();
    //     env::set_current_dir(temp.path()).unwrap();
    //     temp.close().unwrap(); //delete current directory

    //     let repo_path = maybe_get_local_repo();

    //     // set up XDG and Global configs
    //     assert!(repo_path.is_none());
    // }

    #[test]
    fn test_get_local_repo_empty() {
        initialise();
        cd_home();

        let repo_path = maybe_get_local_repo();

        assert!(repo_path.is_none());
    }

    #[test]
    fn test_get_local_repo_with_repo() {
        initialise();
        cd_repo();

        let repo_path = maybe_get_local_repo();

        assert!(repo_path.is_some());
    }

    // -- maybe_open_local_config --

    // #[test]
    // fn test_open_local_config_no_cwd() {
    //     initialise();
    //     let temp = assert_fs::TempDir::new().unwrap();
    //     env::set_current_dir(temp.path()).unwrap();
    //     temp.close().unwrap(); //delete current directory

    //     let no_cwd = maybe_open_local_config();

    //     assert!(no_cwd.get_string("gitlab.host").is_err());
    //     assert!(no_cwd.get_string("gitlab.token").is_err());
    //     assert!(no_cwd.get_bool("gitlab.tls").is_err());
    // }

    #[test]
    fn test_open_local_config_no_repo() {
        initialise();
        cd_home();

        let no_local_config = maybe_open_local_config();

        assert!(no_local_config.get_string("gitlab.host").is_err());
        assert!(no_local_config.get_string("gitlab.token").is_err());
        assert!(no_local_config.get_bool("gitlab.tls").is_err());
        assert!(no_local_config.get_string("gitlab.format").is_err());
    }

    #[test]
    fn test_open_local_config_no_config() {
        initialise();
        cd_repo();
        std::fs::remove_file(".git/config").unwrap();

        let empty_local_config = maybe_open_local_config();

        assert!(empty_local_config.get_string("gitlab.host").is_err());
        assert!(empty_local_config.get_string("gitlab.token").is_err());
        assert!(empty_local_config.get_bool("gitlab.tls").is_err());
        assert!(empty_local_config.get_string("gitlab.format").is_err());
        reset_repo();
    }

    #[test]
    fn test_open_local_config_empty_repo() {
        initialise();
        cd_repo();

        let empty_local_config = maybe_open_local_config();

        assert!(empty_local_config.get_string("gitlab.host").is_err());
        assert!(empty_local_config.get_string("gitlab.token").is_err());
        assert!(empty_local_config.get_bool("gitlab.tls").is_err());
        assert!(empty_local_config.get_string("gitlab.format").is_err());
    }

    #[test]
    fn test_open_local_config_nonempty_repo() {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_bool("gitlab.tls", true).unwrap();
        git_config.set_str("gitlab.format", "json").unwrap();
        cd_repo();
        let nonempty_local_config = maybe_open_local_config();

        assert_eq!(nonempty_local_config.get_string("gitlab.token").unwrap(), "testtoken");
        assert_eq!(nonempty_local_config.get_string("gitlab.host").unwrap(), "some.host.name");
        assert!(nonempty_local_config.get_bool("gitlab.tls").unwrap());
        assert_eq!(nonempty_local_config.get_string("gitlab.format").unwrap(), "json");
        reset_repo();
    }

    // -- update_config_from_git --

    #[test]
    fn test_update_config_from_empty_git() {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let git_config = repo.config().unwrap();
        let mut config = Config::new();
        cd_repo();

        update_config_from_git(&mut config, &git_config);

        assert!(config.token.is_none());
        assert!(config.host.is_none());
        assert!(config.tls.is_none());
        assert!(config.format.is_none());
    }

    #[test]
    fn test_update_config_from_git() {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_bool("gitlab.tls", true).unwrap();
        git_config.set_str("gitlab.format", "json").unwrap();
        let mut config = Config::new();
        cd_repo();

        update_config_from_git(&mut config, &git_config);

        assert_eq!(config.token.unwrap(), "testtoken");
        assert_eq!(config.host.unwrap(), "some.host.name");
        assert_eq!(config.format.unwrap(),OutputFormat::JSON);
        assert!(config.tls.unwrap());
    }

    #[rstest(
        switch,
        case("True"),
        case("TrUe"),
        case("yes"),
        case("YEs"),
        case("1"),
        case("on"),
    )]
    fn test_update_config_from_git_boolean_true_variants(switch: &str) {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_str("gitlab.tls",switch).unwrap();
        git_config.set_str("gitlab.format", "json").unwrap();
        let mut config = Config::new();

        update_config_from_git(&mut config, &git_config);

        assert_eq!(config.token.unwrap(), "testtoken");
        assert_eq!(config.host.unwrap(), "some.host.name");
        assert_eq!(config.format.unwrap(), OutputFormat::JSON);
        assert!(config.tls.unwrap());
    }

    #[rstest(
        switch,
        case("no"),
        case("False"),
        case("FaLse"),
        case("oFF"),
        case("0"),
        case("NO"),
        case("rubbish"),
        case("un dkf sdf sd"),
        case("x"),
    )]
    fn test_update_config_from_git_boolean_false_variants(switch: &str) {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_str("gitlab.tls",switch).unwrap();
        git_config.set_str("gitlab.format", "json").unwrap();
        let mut config = Config::new();

        update_config_from_git(&mut config, &git_config);

        assert_eq!(config.token.unwrap(), "testtoken");
        assert_eq!(config.host.unwrap(), "some.host.name");
        assert_eq!(config.format.unwrap(), OutputFormat::JSON);
        assert!(!config.tls.unwrap());
    }

    // -- get_user_config_type --

    #[test]
    fn test_get_user_config_type_xdg() {
        initialise();
        cd_home();

        let config_type = get_user_config_type();

        assert_eq!(config_type.unwrap(), UserGitConfigLevel::XDG);
    }

    #[test]
    fn test_get_user_config_type_global() {
        initialise();
        // remove the XDG config file first, or this will be picked up instead of Global
        std::fs::remove_file(HOME.child(".config/git/config").path()).unwrap();

        let config_type = get_user_config_type();

        assert_eq!(config_type.unwrap(), UserGitConfigLevel::Global);
        reset_xdg_config();
    }

    // -- get_level_config --

    #[test]
    fn test_get_level_config() {
        initialise();
        let repo = Repository::open("repo").unwrap();
        let git_config = repo.config().unwrap();
        git_config.open_level(XDG).unwrap().set_str("gitlab.token", "xdgtoken").unwrap();
        git_config.open_level(Global).unwrap().set_str("gitlab.token", "globaltoken").unwrap();

        let single_level = get_level_config(&git_config, XDG);
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "xdgtoken");
        let single_level = get_level_config(&git_config, Global);
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "globaltoken");

        reset_global_config();
        reset_xdg_config();
    }

    // -- update_config_from_env --

    #[test]
    fn test_update_config_from_env() {
        initialise();
        let mut conf = Config::new();
        conf.token = Some("token".to_string());

        use std::collections::HashMap;
        let mut env = HashMap::new();

        env.insert("GITLABCLI_TOKEN".to_string(), "env_token".to_string());
        env.insert("GITLABCLI_HOST".to_string(), "env_host".to_string());
        env.insert("GITLABCLI_TLS".to_string(), "yeS".to_string());
        env.insert("GITLABCLI_FORMAT".to_string(), "Json".to_string());

        update_config_from_env(&mut conf, env.into_iter());

        assert_eq!(conf.token.unwrap(), "env_token");
        assert_eq!(conf.host.unwrap(), "env_host");
        assert_eq!(conf.format.unwrap(), OutputFormat::JSON);
        assert!(conf.tls.unwrap());
    }

    // -- test_write_config --

    #[test]
    fn test_write_config() {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();

        let conf = Config {
            token: Some("brad".to_string()),
            host: Some("bradhost".to_string()),
            tls: Some(false),
            format: Some(OutputFormat::JSON),
            projectid: Some(42),
            repo_path: None,
            user_config_type: None
        };

        write_config(&mut git_config, &conf).unwrap();

        assert_eq!(git_config.get_string("gitlab.token").unwrap(), "brad");
        assert_eq!(git_config.get_string("gitlab.host").unwrap(), "bradhost");
        assert_eq!(git_config.get_string("gitlab.projectid").unwrap(), "42");
        assert!(!git_config.get_bool("gitlab.tls").unwrap());
        assert_eq!(git_config.get_string("gitlab.format").unwrap(), "json");

        reset_global_config();
        reset_xdg_config();
        reset_repo();
    }

    #[test]
    #[should_panic(expected = "Failed to save gitlab.host to git config.")]
    fn test_write_config_force_write_error() {
        initialise();
        cd_home();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();

        let conf = Config {
            token: Some("brad".to_string()),
            host: Some("bradhost".to_string()),
            tls: Some(false),
            format: Some(OutputFormat::JSON),
            projectid: Some(42),
            repo_path: None,
            user_config_type: None
        };

        // delete the whole repo
        std::fs::remove_dir_all(HOME.child("repo").path()).unwrap();

        write_config(&mut git_config, &conf).unwrap(); // should panic

        reset_global_config();
        reset_xdg_config();
        reset_repo();
    }

    #[test]
    fn test_write_config_missing_value() {
        initialise();
        cd_home();
        reset_repo();
        let repo = Repository::open("repo").unwrap();
        let mut git_config = repo.config().unwrap();

        let conf = Config {
            token: Some("brad".to_string()),
            host: None,
            tls: Some(false),
            format: Some(OutputFormat::JSON),
            projectid: Some(42),
            repo_path: None,
            user_config_type: None
        };

        write_config(&mut git_config, &conf).unwrap();

        assert_eq!(git_config.get_string("gitlab.token").unwrap(), "brad");
        assert!(git_config.get_string("gitlab.host").is_err());
        assert!(!git_config.get_bool("gitlab.tls").unwrap());

        reset_global_config();
        reset_xdg_config();
        reset_repo();
    }

    // -- Config::save() --

    #[test]
    fn test_save_repo_config() {
        initialise();
        cd_home();
        reset_repo();
        std::fs::remove_file(".config/git/config").unwrap(); //get rid of XDG config file
        let repo = Repository::open("repo").unwrap();
        let git_config = repo.config().unwrap();
        cd_repo();

        // create an empty config with only repo_path and user_config_type = Global
        // the below asserts confirm this.
        let mut conf = Config::defaults();
        assert!(conf.token.is_none());
        assert!(conf.host.is_none());
        assert!(conf.tls.is_none());
        assert!(conf.format.is_none());
        assert!(conf.repo_path.as_ref().unwrap().to_str().unwrap().to_string().starts_with("/tmp/")); //TempDir uses /tmp
        assert_eq!(conf.user_config_type, Some(UserGitConfigLevel::Global));

        // now we change up the conf a bit but not all attributes, so we can test the None case.
        conf.host = Some("testhost".to_string());
        conf.token = Some("test-token".to_string());
        conf.tls = None;
        conf.format = Some(OutputFormat::JSON);

        // now lets try to save to the Local repo config
        conf.save(GitConfigSaveableLevel::Repo).unwrap();

        // now we read it back in to assert
        let mut single_level = get_level_config(&git_config, Local);
        assert_eq!(single_level.get_entry("gitlab.host").unwrap().value().unwrap(), "testhost");
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "test-token");
        assert!(single_level.get_entry("gitlab.tls").is_err()); // it should error out looking this up
        assert_eq!(single_level.get_entry("gitlab.format").unwrap().value().unwrap(), "json");

        // lets check Global to make sure it's not there
        single_level = get_level_config(&git_config, Global);
        assert!(single_level.get_entry("gitlab.token").is_err()); // it should error out looking this up

        reset_global_config();
        reset_xdg_config();
        reset_repo();
    }

    #[test]
    fn test_save_user_global_config() {
        initialise();
        cd_home();
        reset_repo();
        std::fs::remove_file(".config/git/config").unwrap(); //get rid of XDG config file
        let repo = Repository::open("repo").unwrap();
        let git_config = repo.config().unwrap();

        // create an empty in-house config with only user_config_type = Global
        // the below asserts confirm this.
        let mut conf = Config::defaults();
        assert!(conf.token.is_none());
        assert!(conf.host.is_none());
        assert!(conf.tls.is_none());
        assert!(conf.format.is_none());
        assert!(conf.repo_path.is_none()); // not set unless we're in a repo
        assert_eq!(conf.user_config_type, Some(UserGitConfigLevel::Global));

        // now we change up the conf a bit but not all attributes, so we can test the None case.
        conf.host = Some("testhost-globalxxx".to_string());
        conf.token = Some("test-token-globalxxx".to_string());
        conf.tls = None;
        conf.format = Some(OutputFormat::JSON);

        conf.save(GitConfigSaveableLevel::User).unwrap();

        // now we read it back in to assert
        let mut single_level = get_level_config(&git_config, Global);
        assert_eq!(single_level.get_entry("gitlab.host").unwrap().value().unwrap(), "testhost-globalxxx");
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "test-token-globalxxx");
        assert!(single_level.get_entry("gitlab.tls").is_err()); // it should error out looking this up
        assert_eq!(single_level.get_entry("gitlab.format").unwrap().value().unwrap(), "json");

        // lets check Local to make sure it's not there
        single_level = get_level_config(&git_config, Local);
        assert!(single_level.get_entry("gitlab.token").is_err()); // it should error out looking this up

        reset_global_config();
        reset_xdg_config();
        reset_repo();
    }

    #[test]
    fn test_save_user_xdg_config() {
        initialise();
        cd_home();
        reset_repo();
        let repo = Repository::open("repo").unwrap();
        let git_config = repo.config().unwrap();

        // create an empty in-house config with only repo_path and user_config_type = Global
        // the below asserts confirm this.
        let mut conf = Config::defaults();
        println!("{:#?}", &conf);
        assert!(conf.token.is_none());
        assert!(conf.host.is_none());
        assert!(conf.tls.is_none());
        assert!(conf.format.is_none());
        assert!(conf.repo_path.is_none()); // not set unless we're in a repo
        assert_eq!(conf.user_config_type, Some(UserGitConfigLevel::XDG));

        // now we change up the conf a bit but not all attributes, so we can test the None case.
        conf.host = Some("testhost-xdg".to_string());
        conf.token = Some("test-token-xdg".to_string());
        conf.tls = None;
        conf.format = Some(OutputFormat::JSON);

        conf.save(GitConfigSaveableLevel::User).unwrap();

        // now we read it back in to assert
        let mut single_level = get_level_config(&git_config, XDG);
        assert_eq!(single_level.get_entry("gitlab.host").unwrap().value().unwrap(), "testhost-xdg");
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "test-token-xdg");
        assert_eq!(single_level.get_entry("gitlab.format").unwrap().value().unwrap(), "json");

        // lets check Local to make sure it's not there
        single_level = get_level_config(&git_config, Local);
        assert!(single_level.get_entry("gitlab.token").is_err()); // it should error out looking this up

        reset_global_config();
        reset_xdg_config();
        reset_repo();
    }

    // -- Config::defaults() --

    #[test]
    fn test_read_local_config() {
        initialise();
        cd_home();
        reset_repo();
        let repo = Repository::open("repo").unwrap();
        cd_repo();
        let mut config = repo.config().unwrap();
        config.set_str("gitlab.token", "testtoken").unwrap();
        config.set_str("gitlab.host", "some.host.name").unwrap();
        config.set_str("gitlab.format", "json").unwrap();
        config.set_bool("gitlab.tls", true).unwrap();

        let conf = Config::defaults();

        assert_eq!(conf.token.unwrap(), "testtoken");
        assert_eq!(conf.host.unwrap(), "some.host.name");
        assert_eq!(conf.format.unwrap(), OutputFormat::JSON);
        assert!(conf.tls.unwrap());
    }
}
