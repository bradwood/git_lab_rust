use assert_fs::prelude::*;
use git2::Repository;
use lazy_static::*;
use std::env;
use std::path::Path;
use std::sync::Once;

// This is a single, static TempDir used to run all tests in.
lazy_static! {
    pub static ref HOME: assert_fs::TempDir = assert_fs::TempDir::new().unwrap().into_persistent();
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

pub fn initialise() {
    INIT.call_once(|| {
        env::set_var("HOME", HOME.path());
        std::fs::write(HOME.child(".gitconfig").path(), "").unwrap();

        env::set_var("XDG_CONFIG_HOME", HOME.child(".config").path());
        std::fs::create_dir_all(HOME.child(".config/git").path()).unwrap();
        std::fs::write(HOME.child(".config/git/config").path(), "").unwrap();

        let repo_path = HOME.child("repo");
        Repository::init(repo_path.path()).unwrap();
        cd_home();
    });
}

// -- convenience functions --

pub fn cd_home() {
    env::set_current_dir(Path::new(&HOME.path())).unwrap();
    println!("HOME directory: {}", HOME.path().display())
}

pub fn cd_repo() {
    env::set_current_dir(Path::new(&HOME.path()).join("repo")).unwrap();
}

pub fn reset_repo() {
    if std::path::Path::is_dir(HOME.child("repo").path()) {
        std::fs::remove_dir_all(HOME.child("repo").path()).unwrap();
    }
    let repo_path = HOME.child("repo");
    Repository::init(repo_path.path()).unwrap();
}

pub fn reset_global_config() {
    std::fs::write(HOME.child(".gitconfig").path(), "").unwrap();
}

pub fn reset_xdg_config() {
    std::fs::write(HOME.child(".config/git/config").path(), "").unwrap();
}
