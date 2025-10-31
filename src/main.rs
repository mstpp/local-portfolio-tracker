// #![allow(dead_code)]
use clap::{Parser, Subcommand};
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
}

fn main() {
    let cli = Cli::parse();

    match &cli.commands {
        Cmd::List => {
            portfolio_file::list();
        }
        Cmd::New { name } => {
            portfolio_file::new(name.as_str());
        }
        Cmd::Show { name } => {
            csv::show_trades(name);
        }
        Cmd::Report { name } => {
            report::show_holdings(name);
        }
    }

    // println!("DEBUG: {:?}", cli);
}
