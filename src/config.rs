use git2::Repository;
use std::env;

use crate::utils::find_git_root;

#[derive(Debug)]
pub struct Config {
    apitoken: Option<String>,
    host: Option<String>,
    tls: Option<bool>,
}

impl Config {
    pub fn defaults() -> Config {
        let cwd = env::current_dir().expect("Failed to get current directory");

        let repo_path = match find_git_root(&cwd) {
            Some(p) => p,
            None => panic!("Could not find local git repo"),
        };

        let repo = Repository::open(repo_path).expect("Could not open local repo");

        let git_config = repo.config().expect("Could not find config for local repo");

        let apitoken = match git_config.get_string("gitlab.apitoken") {
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

        Config {
            apitoken,
            host,
            tls,
        }
    }
}

#[cfg(test)]
mod config_unit_tests {
    use super::*;
    // use assert_fs::prelude::*;
    // use predicates::prelude::*;

    use git2::Repository;

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
    #[should_panic(expected = "Could not find local git repo")]
    fn test_no_git_repo() {
        let temp = assert_fs::TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        Config::defaults();
        temp.close().unwrap();
    }

    #[test]
    fn test_empty_git_config() {
        let t = setup_repo();
        let conf = Config::defaults();
        assert_eq!(conf.apitoken, None);
        assert_eq!(conf.host, None);
        assert_eq!(conf.tls, None);
        teardown_repo(t);
    }

    #[test]
    fn test_read_local_config() {
        let t = setup_repo();
        let repo = Repository::open(t.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("gitlab.apitoken", "testtoken").unwrap();
        config.set_str("gitlab.host", "some.host.name").unwrap();
        config.set_bool("gitlab.tls", true).unwrap();

        let conf = Config::defaults();

        assert_eq!(conf.apitoken.unwrap(), "testtoken");
        assert_eq!(conf.host.unwrap(), "some.host.name");
        assert!(conf.tls.unwrap());
        teardown_repo(t);
    }
}
