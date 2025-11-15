use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

mod common;
use common::{stdout_list_has, stdout_no_portfolios};
mod fixtures;
use fixtures::TestContext;

mod help {
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

mod list {
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

mod new {
    use super::*;
    #[test]
    fn create_new_portfolio_then_list_shows_it() {
        let ctx = TestContext::new();
        let name = "alpha";

        ctx.create_portfolio(name);
        ctx.assert_portfolio_exists(name);

        ctx.cmd()
            .arg("list")
            .assert()
            .success()
            .stdout(stdout_list_has(name))
            .stderr(predicate::str::is_empty());
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
                predicate::str::contains("Error: Couldn't create new csv file")
                    .and(predicate::str::contains("File exists")),
            )
            .stdout(predicate::str::is_empty());
    }
}
