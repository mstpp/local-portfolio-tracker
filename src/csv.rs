// #![allow(dead_code)]
use crate::portfolio_file::path_from_name;
use crate::trade::Trade;
use anyhow::Result;

//  v1: anyhow::Error instead of Box<dyn Error>
//  v2: anyhow::Result
pub fn read_trades_from_csv(name: &str) -> Result<Vec<Trade>> {
    let path = path_from_name(name)?;
    // println!("DEBUG: path buf to read trades from: {:?}", &path);
    let file = std::fs::File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);

    // Learning 📖
    // v1
    // let trades_result: Result<Vec<Trade>, csv::Error> = reader.deserialize().collect();
    // trades_result

    // Learning 📖
    // v2
    let trades: Vec<Trade> = reader
        .deserialize() // returns iterator of Result<Trade, csv::Error>
        .collect::<Result<Vec<Trade>, csv::Error>>()?;

    // v3 - with filtering out lines that can't be deserialized 🪲 not a good idea, masking bugs (UTC timestamp)
    // TODO filtering out err lines silently?
    // let trades: Vec<Trade> = reader.deserialize().filter_map(Result::ok).collect();

    // Learning 📖
    // v4 - use validate() for input values - ⛔️ validation moved to deserializer
    // let trades: Vec<Trade> = reader
    //     .deserialize()
    //     .filter_map(|res| match res {
    //         Ok(trade) => match trade.validate() {
    //             Ok(()) => Some(trade),
    //             Err(e) => {
    //                 eprintln!("Skipping invalid trade: {e}");
    //                 None
    //             }
    //         },
    //         Err(e) => {
    //             eprintln!("Skipping malformed row: {e}");
    //             None
    //         }
    //     })
    //     .collect();

    Ok(trades)
}

pub fn show_trades(name: &str) -> Result<()> {
    let trades = read_trades_from_csv(name)?;
    if trades.len() == 0 {
        println!("No trades found in '{}'", &name);
    }
    for trade in trades {
        println!("{:?}", &trade);
    }
    Ok(())
}
