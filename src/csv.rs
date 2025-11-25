// #![allow(dead_code)]
use crate::portfolio_file::path_from_name;
use crate::settings::Settings;
use crate::trade::Trade;
use anyhow::Result;
use std::rc::Rc;

pub fn read_trades_from_csv(name: &str, settings: Rc<Settings>) -> Result<Vec<Trade>> {
    let path = path_from_name(name, settings)?;
    let file = std::fs::File::open(path)?;
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
