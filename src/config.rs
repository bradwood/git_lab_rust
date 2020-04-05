use git2::Repository;
use std::env;

use crate::utils::find_git_root;
use anyhow::{Context, Result};

/// This struct holds the config data required to talk to a GitLab server.
///
/// It exploits the standard git-config(1) mechanics and thus can read this data from:
///  * `/etc/gitconfig` --- the __system__ config
///  * `$HOME/.gitconfig` or `$XDG_CONFIG_HOME/git/config` --- the __global__ config
///  * `<REPO_DIR>/.git/confg` --- the repo-specific or __local__ config
///
/// Overrides go from __local__ to __system__ as you'd expect. The [`init`] command can be used to
/// set up the appropriate configuration.
///
/// [`init`]: ../cmds/init/struct.Init.html
#[derive(Debug)]
pub struct Config {
    token: Option<String>,
    host: Option<String>,
    tls: Option<bool>,
}

impl Config {
    pub fn defaults() -> Result<Config> {
        // open multi-level default config object which includes system, global and XDG
        // open local config object if a git repo is found. Warn (but don't fail) if not found

        // search each config, from the top down, for each config item and add to the struct.

        let cwd = env::current_dir().context("Failed to get current directory")?;
        trace!("Got current working directory {}", &cwd.display());

        let repo_path = find_git_root(&cwd)
            .with_context(|| format!("Could not find git repo in {}", &cwd.display()))?;
        trace!("Found nearest git repo at {}", &repo_path.display());

        let repo = Repository::open(&repo_path)
            .with_context(|| format!("Could not open local repo {}", &repo_path.display()))?;
        trace!("Opened git repo at {}", &repo_path.display());

        let git_config = repo.config().with_context(|| {
            format!(
                "Could not find config for local repo {}",
                &repo_path.display()
            )
        })?;
        trace!("Read git config from repo at {}", &repo_path.display());

        let token = match git_config.get_string("gitlab.token") {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        let host = match git_config.get_string("gitlab.host") {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        let tls = match git_config.get_bool("gitlab.tls") {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        trace!("Returning config");
        Ok(Config { token, host, tls })
    }
}

#[cfg(test)]
mod config_unit_tests {
    use super::*;

    fn setup_repo() -> assert_fs::TempDir {
        let temp = assert_fs::TempDir::new().unwrap();
        Repository::init(temp.path()).unwrap();
        env::set_current_dir(temp.path()).unwrap();
        println!("{}", temp.path().display());
        temp
    }

    fn teardown_repo(r: assert_fs::TempDir) {
        r.close().unwrap();
    }

    #[test]
    // #[should_panic(expected = "Could not find local git repo")]
    fn test_no_git_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        let c = Config::defaults();
        if let Err(e) = c {
            println!("{}", e);
            assert!(e.to_string().contains("Could not find git repo in"));
        };
        temp.close().unwrap();
    }

    #[test]
    fn test_empty_git_config() {
        let t = setup_repo();
        let conf = Config::defaults().unwrap();
        assert_eq!(conf.token, None);
        assert_eq!(conf.host, None);
        assert_eq!(conf.tls, None);
        teardown_repo(t);
    }

    #[test]
    fn test_read_local_config() {
        let t = setup_repo();
        let repo = Repository::open(t.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("gitlab.token", "testtoken").unwrap();
        config.set_str("gitlab.host", "some.host.name").unwrap();
        config.set_bool("gitlab.tls", true).unwrap();

        let conf = Config::defaults().unwrap();

        assert_eq!(conf.token.unwrap(), "testtoken");
        assert_eq!(conf.host.unwrap(), "some.host.name");
        assert!(conf.tls.unwrap());
        teardown_repo(t);
    }
}
