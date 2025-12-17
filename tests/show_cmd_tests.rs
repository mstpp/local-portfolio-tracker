mod common;
mod fixtures;

use crate::fixtures::TestContext;

#[test]
fn show_trades_on_empty_portfolio() {
    let ctx = TestContext::new();
    let name = "empty";

    ctx.create_portfolio(name);
    ctx.assert_portfolio_exists(name);
    ctx.show_empty_portfolio(name);
}
