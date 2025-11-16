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
        let name = "bravo";

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
            )
            .stdout(predicate::str::is_empty());
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
        // ctx.assert_portfolio_exists(name);
        // ctx.show_empty_portfolio(name);
        ctx.cmd()
            .args([
                "add-tx", "--name", name, "--ticker", "BTC/USD", "--side", "BUY", "--qty", "0.2",
                "--price", "96450", "--fee", "2",
            ])
            .assert()
            .success()
            .code(0)
            .stdout(predicate::str::contains(
                "Added transaction to portfolio csv file:",
            ))
            .stderr(predicate::str::is_empty());

        let portfolio_path = ctx.portfolio_path(name);
        let contents = std::fs::read_to_string(&portfolio_path).expect("Can't read portfolio file");
        println!("DEBUG: File contents:\n{}", &contents);

        ctx.cmd()
            .args(["show", "--name", name])
            .assert()
            .success()
            .code(0)
            .stderr(predicate::str::is_empty())
            .stdout(predicate::str::contains("created_at"));
    }
}
