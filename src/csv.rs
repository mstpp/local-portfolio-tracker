// #![allow(dead_code)]
use crate::portfolio_file::path_str_from_name;
use crate::trade::Trade;
use anyhow::Result;

//  v1: anyhow::Error instead of Box<dyn Error>
//  v2: anyhow::Result
pub fn read_trades_from_csv(path: &str) -> Result<Vec<Trade>> {
    let file = std::fs::File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);

    // Learning 📖
    // v1
    // let trades_result: Result<Vec<Trade>, csv::Error> = reader.deserialize().collect();
    // trades_result

    // Learning 📖
    // v2
    // let trades: Vec<Trade> = reader
    //     .deserialize() // returns iterator of Result<Trade, csv::Error>
    //     .collect::<Result<Vec<Trade>, csv::Error>>()?;

    // v3 - with filtering out lines that can't be deserialized
    // TODO filtering out err lines silently?
    let trades: Vec<Trade> = reader.deserialize().filter_map(Result::ok).collect();

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
    let path = path_str_from_name(name)?;
    let trades = read_trades_from_csv(&path)?;
    for trade in trades {
        println!("{:?}", &trade);
    }
    Ok(())
}
