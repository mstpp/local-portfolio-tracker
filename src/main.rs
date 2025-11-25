// #![allow(dead_code)]
use anyhow::Result;
use clap::{Parser, Subcommand, builder::ValueParser};
use rust_decimal::Decimal;
use settings::Settings;
mod add_tx;
mod csv;
mod portfolio_file;
mod quote;
mod quote_currency;
mod report;
mod settings;
mod trade;
mod trading_pair;

/// CSV Portfolio Tracker
///
/// A command-line tool to manage CSV-based investment portfolios, calculate PnL,
/// and generate performance reports.
#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[command(subcommand)]
    commands: Cmd,
    #[arg(short, long)]
    portfolio_dir: Option<String>,
}

#[derive(Debug, Clone, Subcommand)]
enum Cmd {
    /// List all portfolios
    #[command(visible_aliases = ["l", "ls"])]
    List,
    /// Create new portfolio
    #[command(alias = "n")]
    New {
        #[arg(short, long)]
        name: String,
    },
    /// Show all transactions from portfolio
    #[command(alias = "s")]
    Show {
        #[arg(short, long)]
        name: String,
    },
    /// Report portfolio PnL
    #[command(alias = "r")]
    Report {
        #[arg(short, long)]
        name: String,
    },
    /// Add transaction to portfolio
    AddTx {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        ticker: String,
        #[arg(long)]
        side: String, // BUY or SELL
        #[arg(short, long, value_parser = ValueParser::new(Decimal::from_str_exact))]
        qty: Decimal,
        #[arg(short, long, value_parser = ValueParser::new(Decimal::from_str_exact))]
        price: Decimal,
        #[arg(short, long, value_parser = ValueParser::new(Decimal::from_str_exact))]
        fee: Decimal,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let settings = std::rc::Rc::new(Settings::load(&cli)?);

    match &cli.commands {
        Cmd::List => {
            portfolio_file::print_list(settings)?;
        }
        Cmd::New { name } => {
            portfolio_file::new(name.as_str(), settings)?;
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
            add_tx::add_tx(name, ticker, side, *qty, *price, *fee, settings)?;
        }
    }

    Ok(())
}
