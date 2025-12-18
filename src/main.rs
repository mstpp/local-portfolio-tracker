use anyhow::Result;
use clap::Parser;
use portfolio_tracker::cli::{Cli, Cmd};
use portfolio_tracker::portfolio;
use portfolio_tracker::settings::Settings;
use portfolio_tracker::trade;
use std::cell::RefCell;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut settings: RefCell<Settings> = RefCell::new(Settings::load(&cli)?);

    match &cli.commands {
        Cmd::List => {
            portfolio::list_csv_files(&settings.borrow())?;
        }
        Cmd::New { name, currency } => {
            if let Some(curr) = currency {
                settings.get_mut().update_base_currency(curr)?;
            }
            portfolio::new(name.as_str(), &settings.borrow())?;
        }
        Cmd::Show { name } => {
            portfolio::show_trades(name, &settings.borrow())?;
        }
        Cmd::Report { name } => {
            portfolio::Portfolio::print_unrealized_pnl(
                settings.borrow().path_for(name),
                settings.borrow().base_currency.id.as_str(),
            )?;
        }
        Cmd::AddTx {
            name,
            ticker,
            side,
            qty,
            price,
            fee,
        } => {
            trade::tx_to_csv(name, ticker, side, *qty, *price, *fee, &settings.borrow())?;
        }
    }

    Ok(())
}
