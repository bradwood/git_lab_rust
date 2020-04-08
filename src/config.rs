use git2::Config as GitConfig;
use git2::ConfigLevel;
use git2::ConfigLevel::{Global, Local, System, XDG};
use git2::Repository;
use std::env;

use crate::utils::find_git_root;

/// This struct holds the config data required to talk to a GitLab server.
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
pub struct Config {
    token: Option<String>,
    host: Option<String>,
    tls: Option<bool>,
}

fn update_configs(config: &mut Config, git_config: &GitConfig) {
    for entry in &git_config.entries(Some("gitlab")).unwrap() {
        let entry = entry.unwrap();
        match entry.name().unwrap() {
            "gitlab.token" => config.token = Some(entry.value().unwrap().to_string()),
            "gitlab.host" => config.host = Some(entry.value().unwrap().to_string()),
            "gitlab.tls" => config.tls = Some(entry.value().unwrap().to_uppercase() == "TRUE"),
            _ => (),
        };
        trace!(
            "L: {:?} : {} => {}",
            config,
            entry.name().unwrap(),
            entry.value().unwrap()
        );
    }
}

impl Config {
    pub fn defaults() -> Config {
        let mut config = Config {
            token: None,
            host: None,
            tls: None,
        };

        // open multi-level default config object which includes system, global and XDG configs,
        // but not local. Needed to provide sane behaviour outside of a local git repo.
        let default_config = match GitConfig::open_default() {
            Ok(x) => {
                trace!("Opened multi-level default config");
                x
            }
            Err(_) => {
                warn!("Didn't find default git config, ignoring");
                GitConfig::new().unwrap()
            }
        };

        // search each config, from the top down, for each config item and add to the struct.
        static LEVELS: [ConfigLevel; 3] = [System, XDG, Global];
        for level in LEVELS.iter() {
            let level_config = match default_config.open_level(*level) {
                Ok(x) => {
                    trace!("Opened config at level {:?}", level);
                    x
                }
                Err(_) => {
                    trace!( "Didn't find config at level {:?}, proceeding without", level);
                    GitConfig::new().unwrap()
                }
            };

            update_configs(&mut config, &level_config);
        }

        // open local config object if a git repo is found. If none is found just return an empty
        // GitConfig
        let local = match env::current_dir() {
            Err(_) => GitConfig::new().unwrap(),
            Ok(cwd) => {
                trace!("Got current directory");
                match find_git_root(&cwd) {
                    None => GitConfig::new().unwrap(),
                    Some(git_path) => {
                        trace!("Found local or upstream git repo");
                        match Repository::open(&git_path) {
                            Err(_) => GitConfig::new().unwrap(),
                            Ok(repo) => {
                                trace!("Opened git repo");
                                match repo.config() {
                                    Err(_) => GitConfig::new().unwrap(),
                                    Ok(config) => {
                                        trace!("Opened git repo's local config");
                                        config.open_level(Local).unwrap()
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        // // Search this global repo and override any earlier assignments
        update_configs(&mut config, &local);

        // then search environment variables for overrides and add them if needed.
        if let Ok(v) = env::var("GITLAB_TOKEN") {
            config.token = Some(v)
        };
        if let Ok(v) = env::var("GITLAB_HOST") {
            config.host = Some(v)
        };
        if let Ok(v) = env::var("GITLAB_TLS") {
            config.tls = Some(v.to_uppercase() == "TRUE")
        };

        config
    }
}

#[cfg(test)]
mod config_unit_tests {
    use super::*;

    #[test]
    fn test_no_current_directory() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        temp.close().unwrap(); //delete current directory

        let conf = Config::defaults();

        assert_eq!(conf.token, None);
        assert_eq!(conf.host, None);
        assert_eq!(conf.tls, None);
    }

    #[test]
    fn test_no_git_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();

        let conf = Config::defaults();

        assert_eq!(conf.token, None);
        assert_eq!(conf.host, None);
        assert_eq!(conf.tls, None);

        temp.close().unwrap();
    }

    #[test]
    fn test_empty_git_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        Repository::init(temp.path()).unwrap();

        let conf = Config::defaults();

        assert_eq!(conf.token, None);
        assert_eq!(conf.host, None);
        assert_eq!(conf.tls, None);

        temp.close().unwrap();
    }

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
