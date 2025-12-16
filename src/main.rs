use anyhow::Result;
use clap::Parser;
use portfolio_tracker::cli::{Cli, Cmd};
use portfolio_tracker::portfolio::Portfolio;
use portfolio_tracker::settings::Settings;
use portfolio_tracker::trade;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let settings = std::rc::Rc::new(Settings::load(&cli)?);

    match &cli.commands {
        Cmd::List => {
            trade::list_csv_files(&settings)?;
        }
        Cmd::New { name } => {
            trade::new(name.as_str(), &settings)?;
        }
        Cmd::Show { name } => {
            trade::show_trades(name, &settings)?; // display only what is in the CSV file
        }
        Cmd::Report { name } => {
            Portfolio::print_unrealized_pnl(settings.path_for(name))?;
        }
        Cmd::AddTx {
            name,
            ticker,
            side,
            qty,
            price,
            fee,
        } => {
            trade::tx_to_csv(name, ticker, side, *qty, *price, *fee, &settings)?;
        }
    }

    Ok(())
}
