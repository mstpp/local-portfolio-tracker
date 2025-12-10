use crate::settings::Settings;
use crate::trade::{Side, Trade, TradingPair};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::rc::Rc;

pub fn tx_to_csv(
    portfolio: &str,
    symbol: &str,
    side: &str,
    qty: Decimal,
    price: Decimal,
    fee: Decimal,
    settings: Rc<Settings>,
) -> Result<()> {
    let tx = Trade {
        created_at: time::OffsetDateTime::now_utc(),
        pair: serde_plain::from_str::<TradingPair>(&symbol).unwrap(),
        side: serde_plain::from_str::<Side>(&side).unwrap(),
        amount: qty,
        price: price,
        fee: fee,
    };

    let path = settings.path_for(portfolio);

    let csv_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect(format!("expecting csv file, but not found: {:?}", &path).as_str());
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(csv_file);
    wrt.serialize(&tx).unwrap();
    println!(
        "✅ Added transaction to portfolio csv file: {:?}\n{:?}",
        path, tx
    );
    Ok(())
}

pub fn read_trades_from_csv(name: &str, settings: Rc<Settings>) -> Result<Vec<Trade>> {
    let path = settings.path_for(name);

    let file = std::fs::File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = csv::Reader::from_reader(file);

    // with filtering out lines that can't be deserialized 🪲 not a good idea, masking bugs (UTC timestamp)
    // let trades: Vec<Trade> = reader.deserialize().filter_map(Result::ok).collect();
    let trades: Vec<Trade> = reader
        .deserialize() // returns iterator of Result<Trade, csv::Error>
        .collect::<Result<Vec<Trade>, csv::Error>>()?;

    Ok(trades)
}

pub fn show_trades(name: &str, settings: Rc<Settings>) -> Result<()> {
    let trades = read_trades_from_csv(name, settings)?;
    if trades.len() == 0 {
        println!("No trades found in '{}'", &name);
    }
    for trade in trades {
        println!("{:?}", &trade);
    }
    Ok(())
}
