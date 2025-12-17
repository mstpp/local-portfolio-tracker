use crate::common::fixtures::TestContext;
use predicates::prelude::*;

#[test]
fn show_trades_on_empty_portfolio() {
    let ctx = TestContext::new();
    let name = "empty";

    ctx.create_portfolio(name);
    ctx.assert_portfolio_exists(name);
    ctx.show_empty_portfolio(name);
}

#[test]
fn show_trades_on_eur_base() {
    let ctx = TestContext::new();
    let name = "empty";
    let data = "# base_currency: EUR
created_at,pair,side,amount,price,fee
1704883200,BTC/EUR,BUY,1.0,40000.00,7.50
";

    ctx.create_eur_portfolio(name, data);
    ctx.assert_portfolio_exists(name);

    let exp = "\
+---------------------------------+---------+------+--------+-------+-----+
| created_at                      | pair    | side | amount | price | fee |
+---------------------------------+---------+------+--------+-------+-----+
| Wed, 10 Jan 2024 10:40:00 +0000 | BTC/EUR | Buy  | 1      | 40000 | 7.5 |
+---------------------------------+---------+------+--------+-------+-----+
";
    ctx.cmd()
        .args(["show", "--name", name])
        .assert()
        .success()
        .code(0)
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::diff(exp));
}
