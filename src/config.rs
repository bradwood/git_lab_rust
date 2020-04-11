use git2::Config as GitConfig;
use git2::ConfigLevel;
use git2::ConfigLevel::{Global, Local, System, XDG};
use git2::Repository;
use std::env;
use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};

use crate::utils::find_git_root;

/// This struct holds the config data required to talk to a GitLab server as well as other
/// configuration data, including the path to the local repo (if any).
///
/// It exploits the standard git-config(1) mechanics and thus can read this data from:
///  * `/etc/gitconfig` --- the __system__ config
///  * `$XDG_CONFIG_HOME/git/config` --- the __XDG__ config
///  * `$HOME/.gitconfig` --- the __global__ config
///  * `$GIT_DIR/.git/confg` --- the repo-specific or __local__ config
///  * The environment variables `GITLAB_HOST`, `GITLAB_TOKEN` and `GITLAB_TLS`
///
/// Override priority increases from top to bottom.
///
#[derive(Debug)]
#[derive(PartialEq)]
pub enum GitConfigSaveableLevel {
    Repo,
    User,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum UserGitConfigLevel {
    XDG,
    Global,
}

#[derive(Debug)]
pub struct Config {
    pub token: Option<String>,
    pub host: Option<String>,
    pub tls: Option<bool>,
    pub repo_path: Option<PathBuf>, //convenience param, not saved with ::save()
    pub user_config_type: Option<UserGitConfigLevel>, //convenience param, not saved with ::save()
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
    trace!("got current directory");
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
    let gitlab_vars = vars.filter(|(k, _)| k.starts_with("GITLAB_"));
    for (key, value) in gitlab_vars {
        if key == "GITLAB_TOKEN" { config.token = Some(value); continue };
        if key == "GITLAB_HOST" { config.host = Some(value); continue };
        if key == "GITLAB_TLS" {
            config.tls = Some(
                value.to_uppercase() == "TRUE" ||
                value.to_uppercase() == "YES" ||
                value.to_uppercase() == "ON" ||
                value.to_uppercase() == "1"
                );
            continue
        };
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
fn get_user_config_type(multi_level_config: &GitConfig) -> Option<UserGitConfigLevel> {
    if multi_level_config.open_level(XDG).is_err() {
        Some(UserGitConfigLevel::Global)
    } else {
        Some(UserGitConfigLevel::XDG)
    }
}

/// Write config data to a git config
fn write_config(save_config: &mut GitConfig, config: &Config) -> Result<()> {

    if config.host.is_some() {
        save_config.set_str("gitlab.host", config.host.as_ref().unwrap())
            .context("Failed to save gitlab.host to git config.")?;
    }

    if config.token.is_some() {
        save_config.set_str("gitlab.token", config.token.as_ref().unwrap())
            .context("Failed to save gitlab.token to git config.")?;
    }

    if config.tls.is_some() {
        save_config.set_bool("gitlab.tls", config.tls.unwrap())
            .context("Failed to save gitlab.tls to git config.")?;
    }

    Ok(())
}

impl Config {

    /// Create an empty config object.
    fn new() -> Config {
        Config { token: None, host: None, tls: None, repo_path: None , user_config_type: None}
    }

    /// Reads the configs from the various GitLab sections in the various git config files and
    /// loads them into the Config struct.
    pub fn defaults() -> Config {
        let mut config = Self::new();

        // Get a local repo if one is there
        config.repo_path = maybe_get_local_repo();

        // Open multi-level default config object which includes system, global and XDG configs,
        // but not local. Needed to provide sane behaviour outside of a local git repo.
        let default_config = maybe_open_multilevel_config();

        // Is the user using a Global ($HOME/.gitconfig) or XDG ($HOME/.config/git/config) style of
        // user level config?
        config.user_config_type = get_user_config_type(&default_config);

        // Update the config from each level of the multi-level-config
        static LEVELS: [ConfigLevel; 3] = [System, XDG, Global];
        #[allow(clippy::suspicious_map)] //using count() below to force iterator consumption
        LEVELS.iter()
            .map(|l|
                update_config_from_git(&mut config,
                    &get_level_config(&default_config, *l)
                    )
                )
            .count();


        // Open local config object if a repo is found else return an empty GitConfig
        let local = maybe_open_local_config();

        // Override any earlier assignments in the struct from the local git config
        update_config_from_git(&mut config, &local);

        // Then update the config from environment variable overrides
        update_config_from_env(&mut config, env::vars());

        config
    }

    /// Saves the config to the appropriate config file. NOTE it will silently apply XDG
    /// instead of Global if config.user_config_type is set to XDG, and vice versa.
    pub fn save(&self, level:GitConfigSaveableLevel) -> Result<()>{
        match level {
            GitConfigSaveableLevel::Repo => {
                let mut save_config = maybe_open_local_config();
                self.repo_path.as_ref().ok_or_else(|| anyhow!("Cannot save to local git repo config if it can't be found."))?;
                write_config(&mut save_config, self)?;
            },
            GitConfigSaveableLevel::User => {
                match self.user_config_type.as_ref().unwrap() {
                    UserGitConfigLevel::Global => {
                        let mut save_config = GitConfig::open_default()
                            .unwrap()
                            .open_level(Global)
                            .unwrap();
                        write_config(&mut save_config, self)?;
                    },
                    UserGitConfigLevel::XDG => {
                        let mut save_config = GitConfig::open_default()
                            .unwrap()
                            .open_level(XDG)
                            .unwrap();
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
    use super::*;
    use rstest::*;
    use assert_fs::prelude::*;

    // -- maybe_get_local_repo --

    #[test]
    fn test_get_local_repo_no_cwd() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        temp.close().unwrap(); //delete current directory

        let repo_path = maybe_get_local_repo();

        assert!(repo_path.is_none());
    }

    #[test]
    fn test_get_local_repo_empty() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();

        let repo_path = maybe_get_local_repo();

        assert!(repo_path.is_none());

        temp.close().unwrap()
    }

    #[test]
    fn test_get_local_repo_with_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        Repository::init(temp.path()).unwrap();

        let repo_path = maybe_get_local_repo();

        assert!(repo_path.is_some());

        temp.close().unwrap()
    }

    // -- maybe_open_local_config --

    #[test]
    fn test_open_local_config_no_cwd() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        temp.close().unwrap(); //delete current directory

        let no_cwd = maybe_open_local_config();

        assert!(no_cwd.get_string("gitlab.host").is_err());
        assert!(no_cwd.get_string("gitlab.token").is_err());
        assert!(no_cwd.get_bool("gitlab.tls").is_err());
    }

    #[test]
    fn test_open_local_config_no_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();

        let no_local_config = maybe_open_local_config();

        assert!(no_local_config.get_string("gitlab.host").is_err());
        assert!(no_local_config.get_string("gitlab.token").is_err());
        assert!(no_local_config.get_bool("gitlab.tls").is_err());

        temp.close().unwrap()
    }

    #[test]
    fn test_open_local_config_no_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        Repository::init(temp.path()).unwrap();
        std::fs::remove_file(".git/config").unwrap();

        let empty_local_config = maybe_open_local_config();

        assert!(empty_local_config.get_string("gitlab.host").is_err());
        assert!(empty_local_config.get_string("gitlab.token").is_err());
        assert!(empty_local_config.get_bool("gitlab.tls").is_err());

        temp.close().unwrap()
    }

    #[test]
    fn test_open_local_config_empty_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        Repository::init(temp.path()).unwrap();

        let empty_local_config = maybe_open_local_config();

        assert!(empty_local_config.get_string("gitlab.host").is_err());
        assert!(empty_local_config.get_string("gitlab.token").is_err());
        assert!(empty_local_config.get_bool("gitlab.tls").is_err());

        temp.close().unwrap()
    }

    #[test]
    fn test_open_local_config_nonempty_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_bool("gitlab.tls", true).unwrap();

        let nonempty_local_config = maybe_open_local_config();

        assert_eq!(nonempty_local_config.get_string("gitlab.token").unwrap(), "testtoken");
        assert_eq!(nonempty_local_config.get_string("gitlab.host").unwrap(), "some.host.name");
        assert!(nonempty_local_config.get_bool("gitlab.tls").unwrap());

        temp.close().unwrap()
    }

    // -- update_config_from_git --

    #[test]
    fn test_update_config_from_empty_git() {
        let git_config = GitConfig::new().unwrap();
        let mut config = Config::new();

        update_config_from_git(&mut config, &git_config);

        assert!(config.token.is_none());
        assert!(config.host.is_none());
        assert!(config.tls.is_none());
    }

    #[test]
    fn test_update_config_from_git() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_bool("gitlab.tls", true).unwrap();
        let mut config = Config::new();

        update_config_from_git(&mut config, &git_config);

        assert_eq!(config.token.unwrap(), "testtoken");
        assert_eq!(config.host.unwrap(), "some.host.name");
        assert!(config.tls.unwrap());

        temp.close().unwrap()
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
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_str("gitlab.tls",switch).unwrap();
        let mut config = Config::new();

        update_config_from_git(&mut config, &git_config);

        assert_eq!(config.token.unwrap(), "testtoken");
        assert_eq!(config.host.unwrap(), "some.host.name");
        assert!(config.tls.unwrap());

        temp.close().unwrap()
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
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.set_str("gitlab.token", "testtoken").unwrap();
        git_config.set_str("gitlab.host", "some.host.name").unwrap();
        git_config.set_str("gitlab.tls",switch).unwrap();
        let mut config = Config::new();

        update_config_from_git(&mut config, &git_config);

        assert_eq!(config.token.unwrap(), "testtoken");
        assert_eq!(config.host.unwrap(), "some.host.name");
        assert!(!config.tls.unwrap());

        temp.close().unwrap()
    }

    // -- get_user_config_type --

    #[test]
    fn test_get_user_config_type() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.open_level(System).unwrap().set_str("gitlab.token", "systemtoken").unwrap();
        git_config.open_level(Global).unwrap().set_str("gitlab.token", "globaltoken").unwrap();

        let mut config_type = get_user_config_type(&git_config);

        assert_eq!(config_type.unwrap(), UserGitConfigLevel::Global);

        git_config.add_file(temp.child("xdg").path(), XDG, true).unwrap();
        git_config.open_level(XDG).unwrap().set_str("gitlab.token", "xdgtoken").unwrap();

        config_type = get_user_config_type(&git_config);

        assert_eq!(config_type, Some(UserGitConfigLevel::XDG));

        temp.close().unwrap()
    }

    // -- get_level_config --

    #[test]
    fn test_get_level_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.open_level(System).unwrap().set_str("gitlab.token", "systemtoken").unwrap();
        git_config.open_level(Global).unwrap().set_str("gitlab.token", "globaltoken").unwrap();

        let single_level = get_level_config(&git_config, System);
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "systemtoken");
        let single_level = get_level_config(&git_config, Global);
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "globaltoken");

        temp.close().unwrap()
    }

    // -- update_config_from_env --

    #[test]
    fn test_update_config_from_env() {
        let mut conf = Config::new();
        conf.token = Some("token".to_string());

        use std::collections::HashMap;
        let mut env = HashMap::new();

        env.insert("GITLAB_TOKEN".to_string(), "env_token".to_string());
        env.insert("GITLAB_HOST".to_string(), "env_host".to_string());
        env.insert("GITLAB_TLS".to_string(), "yeS".to_string());

        update_config_from_env(&mut conf, env.into_iter());

        assert_eq!(conf.token.unwrap(), "env_token");
        assert_eq!(conf.host.unwrap(), "env_host");
        assert!(conf.tls.unwrap());
    }

    // -- test_write_config --

    #[test]
    fn test_write_config() -> Result<()>{
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.add_file(temp.child("xdg").path(), XDG, true).unwrap();

        let conf = Config {
            token: Some("brad".to_string()),
            host: Some("bradhost".to_string()),
            tls: Some(false),
            repo_path: None,
            user_config_type: None
        };

        write_config(&mut git_config, &conf)?;

        assert_eq!(git_config.get_string("gitlab.token").unwrap(), "brad");
        assert_eq!(git_config.get_string("gitlab.host").unwrap(), "bradhost");
        assert!(!git_config.get_bool("gitlab.tls").unwrap());

        temp.close().unwrap();
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Failed to save gitlab.host to git config.")]
    fn test_write_config_force_write_error() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.add_file(temp.child("xdg").path(), XDG, true).unwrap();

        let conf = Config {
            token: Some("brad".to_string()),
            host: Some("bradhost".to_string()),
            tls: Some(false),
            repo_path: None,
            user_config_type: None
        };

        temp.close().unwrap(); // delete directory out from under app

        write_config(&mut git_config, &conf).unwrap();

    }

    #[test]
    fn test_write_config_missing_value() -> Result<()>{
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.add_file(temp.child("xdg").path(), XDG, true).unwrap();

        let conf = Config {
            token: Some("brad".to_string()),
            host: None,
            tls: Some(false),
            repo_path: None,
            user_config_type: None
        };

        write_config(&mut git_config, &conf)?;

        assert_eq!(git_config.get_string("gitlab.token").unwrap(), "brad");
        assert!(git_config.get_string("gitlab.host").is_err());
        assert!(!git_config.get_bool("gitlab.tls").unwrap());

        temp.close().unwrap();
        Ok(())
    }

    // -- Config::save() --

    #[test]
    fn test_save_repo_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.add_file(temp.child("xdg").path(), XDG, true).unwrap();

        // create an empty config with only repo_path and user_config_type = Global
        // the below 5 asserts confirm this.
        let mut conf = Config::defaults();
        assert!(conf.token.is_none());
        assert!(conf.host.is_none());
        assert!(conf.tls.is_none());
        assert!(conf.repo_path.as_ref().unwrap().to_str().unwrap().to_string().starts_with("/tmp/")); //TempDir uses /tmp
        assert_eq!(conf.user_config_type, Some(UserGitConfigLevel::Global));

        // now we change up the conf a bit but not all attributes, so we can test the None case.
        conf.host = Some("testhost".to_string());
        conf.token = Some("test-token".to_string());
        conf.tls = None;

        // println!("{:#?}", &conf);
        // panic!();

        // now lets try to save to the Local repo config
        conf.save(GitConfigSaveableLevel::Repo).unwrap();

        // now we read it back in to assert
        let mut single_level = get_level_config(&git_config, Local);
        assert_eq!(single_level.get_entry("gitlab.host").unwrap().value().unwrap(), "testhost");
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "test-token");

        // lets check Global to make sure it's not there
        single_level = get_level_config(&git_config, Global);
        assert!(single_level.get_entry("gitlab.token").is_err()); // it should error out looking this up

        temp.close().unwrap()
    }

    #[test]
    #[ignore]
    fn test_save_user_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut git_config = repo.config().unwrap();
        git_config.add_file(temp.child("system").path(), System, true).unwrap();
        git_config.add_file(temp.child("global").path(), Global, true).unwrap();
        git_config.add_file(temp.child("xdg").path(), XDG, true).unwrap();

        // create an empty config with only repo_path and user_config_type = Global
        // the below 5 asserts confirm this.
        let mut conf = Config::defaults();
        assert!(conf.token.is_none());
        assert!(conf.host.is_none());
        assert!(conf.tls.is_none());
        assert!(conf.repo_path.as_ref().unwrap().to_str().unwrap().to_string().starts_with("/tmp/")); //TempDir uses /tmp
        assert_eq!(conf.user_config_type, Some(UserGitConfigLevel::Global));

        // now we change up the conf a bit but not all attributes, so we can test the None case.
        conf.host = Some("testhost-sgc".to_string());
        conf.token = Some("test-token-sgc".to_string());
        conf.tls = None;

        // println!("{:#?}", &conf);
        // panic!();

        // TODO: create integration test for this case
        // now lets try to save to the Global repo config
        // CAN'T stub this out. It writes to the _real_ user's config. To be handled later in
        // integration tests
        // conf.save(GitConfigSaveableLevel::User).unwrap();

        // now we read it back in to assert
        let mut single_level = get_level_config(&git_config, Global);
        assert_eq!(single_level.get_entry("gitlab.host").unwrap().value().unwrap(), "testhost-sgc");
        assert_eq!(single_level.get_entry("gitlab.token").unwrap().value().unwrap(), "test-token-sgc");

        // lets check Local to make sure it's not there
        single_level = get_level_config(&git_config, Local);
        assert!(single_level.get_entry("gitlab.token").is_err()); // it should error out looking this up

        temp.close().unwrap()
    }

    // -- Config::defaults() --

    #[test]
    fn test_read_local_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("gitlab.token", "testtoken").unwrap();
        config.set_str("gitlab.host", "some.host.name").unwrap();
        config.set_bool("gitlab.tls", true).unwrap();

        let conf = Config::defaults();

        assert_eq!(conf.token.unwrap(), "testtoken");
        assert_eq!(conf.host.unwrap(), "some.host.name");
        assert!(conf.tls.unwrap());

        temp.close().unwrap();
    }
}
