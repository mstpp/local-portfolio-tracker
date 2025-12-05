use anyhow::Result;
use clap::Parser;
use portfolio_tracker::cli::{Cli, Cmd};
use portfolio_tracker::csv;
use portfolio_tracker::currency::init_tickers_from_csv;
use portfolio_tracker::report;
use portfolio_tracker::settings::Settings;
use std::{path::PathBuf, str::FromStr};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let settings = std::rc::Rc::new(Settings::load(&cli)?);
    // TODO add tickers_data to settings
    init_tickers_from_csv(PathBuf::from_str("./data/coingecko.csv")?)?;

    match &cli.commands {
        Cmd::List => {
            csv::print_list(settings)?;
        }
        Cmd::New { name } => {
            csv::new(name.as_str(), settings)?;
        }
        Cmd::Show { name } => {
            csv::show_trades(name, settings)?; // display only what is in the CSV file
        }
        Cmd::Report { name } => {
            report::show_holdings(name, settings)?;
        }
        Cmd::AddTx {
            name,
            ticker,
            side,
            qty,
            price,
            fee,
        } => {
            csv::tx_to_csv(name, ticker, side, *qty, *price, *fee, settings)?;
        }
    }

    Ok(())
}
