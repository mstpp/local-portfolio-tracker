// #![allow(dead_code)]
use clap::{Parser, Subcommand};
mod add_tx;
mod csv;
mod portfolio_file;
mod quote;
mod report;
mod trade;
mod trading_pair;

/// CSV Portfolio Tracker
///
/// A command-line tool to manage CSV-based investment portfolios, calculate PnL,
/// and generate performance reports.
#[derive(Debug, Clone, Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Cmd,
}

#[derive(Debug, Clone, Subcommand)]
enum Cmd {
    /// List all portfolios located at ./portfolios
    List,
    /// Create new portfolio at ./portfolios
    New { name: String },
    /// Show all transactino from portfolio
    Show { name: String },
    /// Report portfolio PnL total and per asset
    Report { name: String },
    /// Add transaction to portfolio
    #[command(subcommand)]
    AddTx(AddTxCmd),
}

#[derive(Debug, Clone, Subcommand)]
enum AddTxCmd {
    /// Add a trade transaction (BUY or SELL)
    Trade {
        portfolio: String,
        symbol: String,
        side: String, // BUY or SELL
        qty: f64,
        price: f64,
        fee: f64,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.commands {
        Cmd::List => {
            let _ = portfolio_file::print_list();
        }
        Cmd::New { name } => {
            let _ = portfolio_file::new(name.as_str());
        }
        Cmd::Show { name } => {
            csv::show_trades(name);
        }
        Cmd::Report { name } => {
            report::show_holdings(name);
        }
        Cmd::AddTx(add_tx_cmd) => match add_tx_cmd {
            AddTxCmd::Trade {
                portfolio,
                symbol,
                side,
                qty,
                price,
                fee,
            } => add_tx::add_tx(portfolio, symbol, side, *qty, *price, *fee),
        },
    }
}
