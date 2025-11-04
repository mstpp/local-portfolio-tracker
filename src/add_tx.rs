use crate::{
    portfolio_file::path_from_name,
    trade::{Side, Trade},
    trading_pair::TradingPair,
};
use anyhow::Result;
use rust_decimal::Decimal;
pub fn add_tx(
    portfolio: &str,
    symbol: &str,
    side: &str,
    qty: Decimal,
    price: Decimal,
    fee: Decimal,
) -> Result<()> {
    let tx = Trade {
        created_at: time::OffsetDateTime::now_utc(),
        pair: serde_plain::from_str::<TradingPair>(&symbol).unwrap(),
        side: serde_plain::from_str::<Side>(&side).unwrap(),
        amount: qty,
        price: price,
        fee: fee,
    };

    let path = path_from_name(portfolio)?;

    let csv_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect(format!("expecting csv file, but not found: {:?}", &path).as_str());
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(csv_file);
    wrt.serialize(&tx).unwrap();
    // wrt.flush().unwrap(); // redundant
    println!(
        "✅ Added transaction to portfolio csv file: {:?}\n{:?}",
        path, tx
    );
    Ok(())
}
