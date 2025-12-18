use crate::common::fixtures::TestContext;
use predicates::prelude::*;

#[test]
fn create_new_portfolio_then_list_shows_it() {
    let ctx = TestContext::new();
    let name = "alpha";

    ctx.create_portfolio(name);
    ctx.assert_portfolio_exists(name);
    ctx.assert_list_contains(name);

    let expected = "\
# base_currency: USD
created_at,pair,side,amount,price,fee
";
    let p_path = ctx.portfolio_path(name);
    let p_content = std::fs::read_to_string(p_path).unwrap();
    pretty_assertions::assert_eq!(expected, p_content.as_str());
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

#[test]
fn create_eur_portfolio() {
    let ctx = TestContext::new();
    let name = "ojro";
    ctx.cmd()
        .args(["new", "--name", name, "--currency", "eur"])
        .assert()
        .success()
        .code(0)
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("Created trades file:"));

    ctx.assert_portfolio_exists(name);

    let expected = "\
# base_currency: EUR
created_at,pair,side,amount,price,fee
";
    let p_path = ctx.portfolio_path(name);
    let p_content = std::fs::read_to_string(p_path).unwrap();
    pretty_assertions::assert_eq!(expected, p_content.as_str());
}
