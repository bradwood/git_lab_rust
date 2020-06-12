use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use serde_json::json;

use crate::config::OutputFormat;

const DOTGIT: &str = ".git";

/// Print out JSON or test based vectors of key/value pairs
pub fn write_short_output<M>(format: Option<OutputFormat>, map: M) -> Result<()>
where
    M: Iterator<Item = (String, String)>
{
    match format {
        Some(OutputFormat::JSON) => {
            let hash: HashMap<_,_> = map.collect();
            let j = json!(&hash);

            println!("{}", j);
            Ok(())
        },
        Some(OutputFormat::Text) | None => {
            for (key, value) in map {
                println!("{}: {}", key, value)
            }
            Ok(())
        }
    }
}

/// Find a git repo in the current directory or any one above it.
pub fn find_git_root(starting_directory: &Path) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let dotgit = Path::new(DOTGIT);

    loop {
        path.push(dotgit);

        if path.is_dir() {
            trace!("Found git root: {:?}", path.as_path().to_str().unwrap());
            break Some(path);
        }

        // remove DOTGIT && remove parent
        if !(path.pop() && path.pop()) {
            trace!("Did not find git root");
            break None;
        }
    }
}

/// various string validators used to ensure clap.rs args pass
pub mod validator {
    use git2::Reference;
    use lazy_static::*;
    use regex::Regex;
    use url::Url;

    /// check for valid u64 int
    pub fn check_u64(v: String) -> Result<(), String> {
        if v.parse::<u64>().is_ok() {
            return Ok(());
        }
        Err(String::from("The value is not a positive integer"))
    }
    /// check for gitlab project slug
    /// Rules:
    /// * can only contain letters, digits, `_`, `-` and `.`
    /// * cannot start with `-`
    /// * cannot end with `.git`
    /// * cannot end with `.atom`
    pub fn check_project_slug(v: String) -> Result<(), String> {
        lazy_static! {
            static ref SLUG_RE: Regex = Regex::new(r"^[A-Za-z0-9_.][A-Za-z0-9-_.]*$").unwrap();
        }
        // below needed as rust doesn't support lookbehind regex
        if SLUG_RE.is_match(&v) && !(v.ends_with(".git") || v.ends_with(".atom")) {
            return Ok(());
        }
        Err(String::from(
            "\nOnly alphanumeric characters, `_`, `-` and `.` permitted\n\
            Cannot start with `-`\n\
            Cannot end with `.git` or `.atom`",
        ))
    }
    /// Checks branch is valid according to git-check-ref-format(1)
    // TODO: Improve this once upstream API changes or bite the bullet and implement it here, but
    // the below should be good enough for most cases.
    // See https://github.com/libgit2/libgit2/issues/5506
    pub fn check_branch_name(v: String) -> Result<(), String> {
        if Reference::is_valid_name(&("refs/heads/".to_owned() + &v)) && !v.starts_with('-') {
            return Ok(());
        }
        Err(String::from("Bad git ref name, see git-check-ref-format(1) for details"))
    }

    /// Check for valid URL
    pub fn check_url(v: String) -> Result<(), String> {
        if Url::parse(&v).is_ok() {
            return Ok(());
        }
        Err(String::from("Bad URL"))
    }
}

#[cfg(test)]
mod validator_unit_tests {
    use super::validator::*;

    #[test]
    fn test_check_url() {
        let v = check_url(String::from("gitlab.com/blah/bah"));
        assert!(v.is_err());
        let v = check_url(String::from("-345"));
        assert!(v.is_err());

        let v = check_url(String::from("https://gitlab.com/blah/bah"));
        assert!(v.is_ok());
        let v = check_url(String::from("http:///1.2.3.4"));
        assert!(v.is_ok());
        let v = check_url(String::from("http://gitlab.com/blah/bah"));
        assert!(v.is_ok());
    }

    #[test]
    fn test_check_u64() {
        let v = check_u64(String::from("brad"));
        assert!(v.is_err());
        let v = check_u64(String::from("-345"));
        assert!(v.is_err());

        let v = check_u64(String::from("345"));
        assert!(v.is_ok());
    }

    #[test]
    fn test_check_project_slug() {
        let v = check_project_slug(String::from("br_ssad-sad0998654678ad"));
        assert!(v.is_ok());
        let v = check_project_slug(String::from("345"));
        assert!(v.is_ok());
        let v = check_project_slug(String::from("_-askdlj"));
        assert!(v.is_ok());

        let v = check_project_slug(String::from("-xx"));
        assert!(v.is_err());
        let v = check_project_slug(String::from("brad="));
        assert!(v.is_err());
        let v = check_project_slug(String::from("brad.atom"));
        assert!(v.is_err());
        let v = check_project_slug(String::from("brad.git"));
        assert!(v.is_err());
    }

    #[test]
    fn test_check_valid_branch_name() {
        let v = check_branch_name(String::from("br_ssad-sad0998654678ad"));
        assert!(v.is_ok());
        let v = check_branch_name(String::from("345"));
        assert!(v.is_ok());
        let v = check_branch_name(String::from("_-askdlj"));
        assert!(v.is_ok());
        let v = check_branch_name(String::from("brad="));
        assert!(v.is_ok());

        let v = check_branch_name(String::from("-xx"));
        assert!(v.is_err());
        let v = check_branch_name(String::from("br@ddf/df/df/"));
        assert!(v.is_err());
        let v = check_branch_name(String::from("//dbrad"));
        assert!(v.is_err());
        let v = check_branch_name(String::from("-brad"));
        assert!(v.is_err());
    }
}
