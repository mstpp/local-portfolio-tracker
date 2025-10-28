// #![allow(dead_code)]
use crate::trade::Trade;
use anyhow::Result;
use std::fs::File;

//  anyhow::Error instead of Box<dyn Error>
pub fn read_trades_from_csv(path: &str) -> Result<Vec<Trade>, anyhow::Error> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);

    // options 1
    // let trades_result: Result<Vec<Trade>, csv::Error> = reader.deserialize().collect();
    // trades_result

    // option 2
    // let trades: Vec<Trade> = reader
    //     .deserialize() // returns iterator of Result<Trade, csv::Error>
    //     .collect::<Result<Vec<Trade>, csv::Error>>()?;

    // option 3 - with filtering out lines that can't be deserialized
    let trades: Vec<Trade> = reader.deserialize().filter_map(Result::ok).collect();

    Ok(trades)
}

pub fn show_trades(name: &str) {
    let path = format!("./portfolios/{}", name);
    let trades = read_trades_from_csv(&path).unwrap();
    for trade in trades {
        println!("{:?}", &trade);
    }
}
