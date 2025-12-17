mod common;
mod fixtures;

use crate::fixtures::TestContext;

#[test]
fn report_single_ticker_single_tx_test() {
    let ctx = TestContext::new();
    let name = "testfolio";
    ctx.create_portfolio(name);
    ctx.add_tx_buy_btc(name, "0.5", "96450", "37");
    ctx.report(name);
}
