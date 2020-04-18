#[cfg(test)]
mod base_integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_cli_no_args() {
        let mut cmd = Command::cargo_bin("git-lab").unwrap();
        let assert = cmd.assert();
        assert
            .success()
            .code(0)
            .stdout(predicate::str::contains("git-lab [FLAGS] [SUBCOMMAND]"));
    }

    // TODO: Add verbosity test!!

    // #[rstest(
    //     switch,
    //     case("-a"),
    //     case("--add"),
    // )]
    // fn test_cli_missing_option_value_fail(switch: &str) {
    //     let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    //     let assert = cmd.arg(switch).assert();
    //     assert.failure().code(1).stderr(predicate::str::contains(
    //         "requires a value but none was supplied",
    //     ));
    // }

    // #[rstest(
    //     switch,
    //     case("-i"),
    //     case("--increase"),
    //     case("-d"),
    //     case("--decrease"),
    // )]
    // fn test_cli_missing_option_value_pass(switch: &str) {
    //     let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    //     let assert = cmd.arg(switch).assert();
    //     assert.success().code(0);
    // }

    // #[rstest(
    //     switch,
    //     case("--help"),
    //     case("--version"),
    //     case("--purge"),
    //     case("--stats"),
    //     case("--complete"),
    // )]
    // fn test_cli_option_without_val(switch: &str) {
    //     let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    //     let assert = cmd.arg(switch).assert();
    //     assert.success().code(0);
    // }

    // #[rstest(
    //     switch,
    //     case("-a"),
    //     case("--add"),
    //     case("-i"),
    //     case("--increase"),
    //     case("-d"),
    //     case("--decrease"),
    // )]
    // fn test_cli_option_with_val(switch: &str) {
    //     let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    //     let assert = cmd.arg(switch).arg("10").assert();
    //     assert.success().code(0);
    // }

    // #[rstest(
    //     switch, param,
    //     case("-i", "1"),
    //     case("-i", "1.02"),
    //     case("-d", "40" ),
    //     case("--decrease", "100000"),
    // )]
    // fn test_cli_check_inc_dec_type_passes(switch: &str, param: &str) {
    //     let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    //     let assert = cmd.arg(switch).arg(param).assert();
    //     assert.success().code(0);
    // }

    // #[rstest(
    //     switch, param,
    //     case("-d", "-2" ),
    //     case("-d", "brad" ),
    //     case("-i", "brad" ),
    //     case("-d", "0" ),
    //     case("-i", "0" ),
    //     case("-i", "0.0" ),
    // )]
    // fn test_cli_check_inc_dec_type_fails(switch: &str, param: &str) {
    //     let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    //     let assert = cmd.arg(switch).arg(param).assert();
    //     assert.failure().code(1);
    // }
}
