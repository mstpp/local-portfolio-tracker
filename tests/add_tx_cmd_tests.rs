mod common;
mod fixtures;

use crate::fixtures::TestContext;
use predicates::prelude::*;

#[test]
fn add_valid_tx_to_new_portfolio() {
    let ctx = TestContext::new();
    let name = "basic";
    ctx.create_portfolio(name);
    ctx.assert_portfolio_exists(name);
    ctx.show_empty_portfolio(name);
    ctx.add_tx_buy_btc(name, "0.5", "96450", "37");

    let expected_stdout_header = "\
+---------------------------------+---------+------+--------+-------+-----+
| created_at                      | pair    | side | amount | price | fee |
+---------------------------------+---------+------+--------+-------+-----+
";
    let expected_stdout_data = " | BTC/USD | Buy  | 0.5    | 96450 | 37  |
+---------------------------------+---------+------+--------+-------+-----+";

    let p = ctx
        .cmd()
        .args(["show", "--name", name])
        .assert()
        .success()
        .code(0)
        .stderr(predicate::str::is_empty())
        .stdout(
            predicate::str::contains(expected_stdout_header)
                .and(predicate::str::contains(expected_stdout_data)),
        );
    // .stdout(
    //     predicate::str::contains("created_at")
    //         .and(predicate::str::contains("pair"))
    //         .and(predicate::str::contains("side"))
    //         .and(predicate::str::contains("amount"))
    //         .and(predicate::str::contains("price"))
    //         .and(predicate::str::contains("fee"))
    //         .and(predicate::str::contains("BTC/USD"))
    //         .and(predicate::str::contains("96450")),
    // );

    println!("DEBUG add_valid_tx_to_new_portfolio:\n\n{p:?}");
}
