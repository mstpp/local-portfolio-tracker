use crate::{
    trade::{Side, Trade},
    trading_pair::TradingPair,
};

pub fn add_tx(portfolio: &str, symbol: &str, side: &str, qty: f64, price: f64, fee: f64) {
    let tx = Trade {
        created_at: time::OffsetDateTime::now_utc(),
        pair: serde_plain::from_str::<TradingPair>(&symbol).unwrap(),
        side: serde_plain::from_str::<Side>(&side).unwrap(),
        amount: rust_decimal::Decimal::from_f64_retain(qty).unwrap(),
        price: rust_decimal::Decimal::from_f64_retain(price).unwrap(),
        fee: rust_decimal::Decimal::from_f64_retain(fee).unwrap(),
    };
    let path = format!("./portfolios/{}", portfolio);

    let csv_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect(format!("expecting csv file, but not found: {}", &path).as_str());
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(csv_file);
    wrt.serialize(&tx).unwrap();
    // wrt.flush().unwrap(); // redundant
    println!(
        "✅ Added transaction to portfolio csv file: {:?}\n{:?}",
        path, tx
    );
}
