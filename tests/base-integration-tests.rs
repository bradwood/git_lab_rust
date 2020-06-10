#[cfg(test)]
mod base_integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_cli_no_args() {
        let mut cmd = Command::cargo_bin("git-lab").unwrap();
        let assert = cmd.assert();
        assert
            .code(1)
            .stderr(predicate::str::contains("custom git command for interacting with a GitLab server"));
    }

}
