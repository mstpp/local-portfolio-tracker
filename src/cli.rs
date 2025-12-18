use clap::{Parser, Subcommand, builder::ValueParser};
use rust_decimal::Decimal;

/// CSV Portfolio Tracker
///
/// A command-line tool to manage CSV-based investment portfolios, calculate PnL,
/// and generate performance reports.
#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Cmd,
    #[arg(short, long)]
    pub portfolio_dir: Option<String>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Cmd {
    /// List all portfolios
    #[command(visible_aliases = ["l", "ls"])]
    List,
    /// Create new portfolio
    #[command(alias = "n")]
    New {
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        currency: Option<String>,
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
