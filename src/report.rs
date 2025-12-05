// #![allow(dead_code)]
use crate::csv::read_trades_from_csv;
use crate::portfolio::Portfolio;
use crate::portfolio_file::path_from_name;
use crate::settings::Settings;
use crate::trade::{Side, Trade};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use rust_decimal::dec;
use rust_decimal::prelude::FromPrimitive;
use std::collections::HashMap;
use std::rc::Rc;

type Position = (Decimal, Decimal, Decimal); // (amount, avg_price, total_fees)
type Book = HashMap<String, Position>;

fn calc_holdings(book: &mut Book, tx: &Trade) {
    let pos = book
        .entry(tx.pair.base.to_string())
        .or_insert((dec!(0), dec!(0), dec!(0)));
    let (amt, prc, f) = *pos;
    let (mut new_amount, mut new_avg, mut new_fee) = (amt, prc, f);

    match tx.side {
        Side::Buy => {
            new_amount += tx.amount;
            new_avg = (amt * prc + tx.amount * tx.price) / new_amount;
        }
        Side::Sell => {
            if tx.amount > amt {
                println!("WARN: Sell amount bigger than holdings. Ignoring this tx.")
            } else {
                new_amount -= tx.amount;
            }
        }
    }
    new_fee += tx.fee;

    *pos = (new_amount, new_avg, new_fee);
}

pub fn show_holdings(name: &str, settings: Rc<Settings>) -> Result<()> {
    let mut holdings: Book = HashMap::new();
    let trades: Vec<Trade> = read_trades_from_csv(&name, settings.clone()).unwrap();
    for tx in trades {
        calc_holdings(&mut holdings, &tx);
    }

    // get all holding tickers
    let tickers: Vec<String> = holdings.clone().into_keys().collect();

    // based on ids, get current quotes
    let quotes_hm = crate::quote::get_quotes(tickers).unwrap();
    for (id, quote) in quotes_hm.clone() {
        println!("{:6}={:10} USD", &id, &quote);
    }

    let mut total_pnl = dec![0];
    for (ticker, price) in quotes_hm {
        let (holding, avg_price, _) = holdings.get(&ticker.clone()).unwrap();
        let dec_price = Decimal::from_f64(price).unwrap();
        let val = holding.clone() * dec_price;
        total_pnl += val;
        let pnl = val - (holding * avg_price);
        let pnl_perc = (pnl / (holding * avg_price)) * dec![100];
        println!(
            "{:6} | holdings {:4}| avg price: {:5}| val: {:10.2} USD| PnL: {:10.2} USD {:6.2}%",
            ticker, holding, avg_price, val, pnl, pnl_perc
        );
    }

    println!();
    println!("Total PnL USD: {total_pnl}");

    // Portfolio processing TODO
    let pathbuf = path_from_name(name, settings).context("Failed to resolve portfolio path")?;
    Portfolio::print_unrealized_pnl(pathbuf)?;

    Ok(())
}
