mod common;

#[cfg(test)]
mod init_integration_tests {
    use assert_cmd::cargo::cargo_bin;
    use assert_fs::prelude::*;
    use crate::common::*;
    use git2::Config as GitConfig;
    use git2::Repository;
    use rexpect::errors::*;
    use rexpect::spawn;


    #[test]
    fn test_init_home_dir() -> Result<()> {
        initialise();
        let cmd_str = format!("{} {}", cargo_bin("git-lab").as_path().to_str().unwrap(), "init");
        println!("cmd: {}", cmd_str);
        let mut p = spawn(&cmd_str, Some(2000))?;
        p.exp_regex("GitLab host \\[.*\\]:")?;
        p.send_line("test-gitlab-host")?;
        p.exp_string("GitLab personal access token:")?;
        p.send_line("test-gitlab-token")?;
        p.exp_regex("TLS enabled \\[.*\\]:")?;
        p.send_line("true")?;
        p.exp_eof()?;

        let git_config = GitConfig::open(&GitConfig::find_xdg().unwrap()).unwrap();
        assert_eq!(git_config.get_entry("gitlab.host").unwrap().value().unwrap(), "test-gitlab-host");
        assert_eq!(git_config.get_entry("gitlab.token").unwrap().value().unwrap(), "test-gitlab-token");
        assert!(git_config.get_bool("gitlab.tls").unwrap());

        reset_global_config();
        reset_xdg_config();
        reset_repo();

        Ok(())
    }

    #[test]
    fn test_init_repo_dir() -> Result<()> {
        initialise();
        cd_repo();
        let cmd_str = format!("{} {}", cargo_bin("git-lab").as_path().to_str().unwrap(), "init");
        println!("cmd: {}", cmd_str);
        let mut p = spawn(&cmd_str, Some(2000))?;
        p.exp_regex("GitLab host \\[.*\\]:")?;
        p.send_line("test-gitlab-host")?;
        p.exp_string("GitLab personal access token:")?;
        p.send_line("test-gitlab-token")?;
        p.exp_regex("TLS enabled \\[.*\\]:")?;
        p.send_line("true")?;
        p.exp_eof()?;

        let repo_path = HOME.child("repo");
        let repo = Repository::init(repo_path.path()).unwrap();
        let git_config = repo.config().unwrap();
        assert_eq!(git_config.get_entry("gitlab.host").unwrap().value().unwrap(), "test-gitlab-host");
        assert_eq!(git_config.get_entry("gitlab.token").unwrap().value().unwrap(), "test-gitlab-token");
        assert!(git_config.get_bool("gitlab.tls").unwrap());

        reset_global_config();
        reset_xdg_config();
        reset_repo();

        Ok(())
    }

    #[test]
    fn test_init_home_dir_from_repo_dir() -> Result<()> {
        initialise();
        cd_repo();
        let cmd_str = format!("{} {}", cargo_bin("git-lab").as_path().to_str().unwrap(), "init --user");
        println!("cmd: {}", cmd_str);
        let mut p = spawn(&cmd_str, Some(2000))?;
        p.exp_regex("GitLab host \\[.*\\]:")?;
        p.send_line("test-gitlab-host")?;
        p.exp_string("GitLab personal access token:")?;
        p.send_line("test-gitlab-token")?;
        p.exp_regex("TLS enabled \\[.*\\]:")?;
        p.send_line("true")?;
        p.exp_eof()?;

        let git_config = GitConfig::open(&GitConfig::find_xdg().unwrap()).unwrap();
        assert_eq!(git_config.get_entry("gitlab.host").unwrap().value().unwrap(), "test-gitlab-host");
        assert_eq!(git_config.get_entry("gitlab.token").unwrap().value().unwrap(), "test-gitlab-token");
        assert!(git_config.get_bool("gitlab.tls").unwrap());

        reset_global_config();
        reset_xdg_config();
        reset_repo();

        Ok(())
    }
}
