use crate::common::fixtures::TestContext;
use predicates::prelude::*;

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
        .stderr(predicate::str::contains("File already exists"));
}
