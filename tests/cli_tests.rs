use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

mod common;
use common::stdout_no_portfolios;
mod fixtures;
use fixtures::TestContext;

mod help_cmd_tests {
    use super::*;

    #[test]
    fn help_flag_displays_usage_information() {
        for flag in ["--help", "-h"] {
            let mut cmd = cargo_bin_cmd!("portfolio-tracker");
            cmd.arg(flag)
                .assert()
                .success() // exit code 0
                .stdout(predicate::str::contains("Usage"))
                .stdout(predicate::str::contains("Commands"))
                .stdout(predicate::str::contains("Options"))
                .stderr(predicate::str::is_empty());
        }
    }
}

mod list_cmd_tests {
    use super::*;

    #[test]
    fn list_on_empty_workspace_shows_no_portfolios() {
        let ctx = TestContext::new();

        ctx.cmd()
            .arg("list")
            .assert()
            .success()
            .stdout(stdout_no_portfolios())
            .stderr(predicate::str::is_empty());
    }
}

mod new_cmd_tests {
    use super::*;
    #[test]
    fn create_new_portfolio_then_list_shows_it() {
        let ctx = TestContext::new();
        let name = "alpha";

        ctx.create_portfolio(name);
        ctx.assert_portfolio_exists(name);
        ctx.assert_list_contains(name);
    }

    #[test]
    fn fail_to_create_duplicate_portfolio() {
        let ctx = TestContext::new();
        let name = "dupli";

        ctx.create_portfolio(name);
        ctx.assert_portfolio_exists(name);

        ctx.cmd()
            .args(["new", "--name", name])
            .assert()
            .failure()
            .code(1)
            .stderr(
                predicate::str::contains("Error while creating csv file")
                    .and(predicate::str::contains("File exists")),
            );
    }
}

mod show_cmd_tests {
    use super::*;

    #[test]
    fn show_trades_on_empty_portfolio() {
        let ctx = TestContext::new();
        let name = "empty";

        ctx.create_portfolio(name);
        ctx.assert_portfolio_exists(name);
        ctx.show_empty_portfolio(name);
    }
}

mod add_tx_cmd_tests {
    use super::*;

    #[test]
    fn add_valid_tx_to_new_portfolio() {
        let ctx = TestContext::new();
        let name = "basic";
        ctx.create_portfolio(name);
        ctx.assert_portfolio_exists(name);
        ctx.show_empty_portfolio(name);
        ctx.add_tx_buy_btc(name, "0.5", "96450", "37");

        ctx.cmd()
            .args(["show", "--name", name])
            .assert()
            .success()
            .code(0)
            .stderr(predicate::str::is_empty())
            .stdout(predicate::str::contains("created_at"));
    }
}

mod report_cmd_tests {
    use super::*;

    #[test]
    fn report_single_ticker_single_tx_test() {
        let ctx = TestContext::new();
        let name = "testfolio";
        ctx.create_portfolio(name);
        ctx.add_tx_buy_btc(name, "0.5", "96450", "37");
        ctx.report(name);
    }
}
