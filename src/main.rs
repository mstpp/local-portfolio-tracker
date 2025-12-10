use anyhow::{Result, anyhow};
use clap::Parser;
use portfolio_tracker::cli::{Cli, Cmd};
use portfolio_tracker::currency::init_tickers_from_csv;
use portfolio_tracker::portfolio::Portfolio;
use portfolio_tracker::settings::Settings;
use portfolio_tracker::trade;
use prettytable::{Table, row};
use std::ffi::OsString;
use std::fs::DirEntry;
use std::time::SystemTime;
use std::{path::PathBuf, str::FromStr};
use time::OffsetDateTime;
use time::macros::format_description;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let settings = std::rc::Rc::new(Settings::load(&cli)?);
    // TODO add tickers_data to settings
    init_tickers_from_csv(PathBuf::from_str("./data/coingecko.csv")?)?;

    match &cli.commands {
        Cmd::List => {
            list_csv_files(&settings)?;
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

// +---------------+---------------------+
// | CSV file name | Created at          |
// +---------------+---------------------+
// | example.csv   | 2025-12-05 20:01:21 |
// +---------------+---------------------+
fn list_csv_files(settings: &Settings) -> Result<()> {
    let mut files: Vec<(OsString, SystemTime)> = Vec::new();

    for entry in settings.portfolio_dir.read_dir()? {
        let entry: DirEntry = entry?;
        let metadata: std::fs::Metadata = entry.metadata()?;

        // Skip directories or special files
        if !metadata.is_file() {
            continue;
        }

        let created = metadata.created().or_else(|_| metadata.modified())?; // fallback for Unix consistency
        let path = entry.path();
        let name = path.file_stem().ok_or(anyhow!("err getting name"))?;
        files.push((name.to_os_string(), created));
    }

    // Sort oldest → newest
    files.sort_unstable_by_key(|(_, t)| *t);

    // pretty table
    let mut table = Table::new();
    table.add_row(row!["CSV file name", "Created at"]);

    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    for (name, timestamp) in &files {
        let created = OffsetDateTime::from(*timestamp);
        table.add_row(row![name.to_string_lossy(), created.format(format)?]);
    }

    table.printstd();

    Ok(())
}
