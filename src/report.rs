use crate::csv::path_from_name;
use crate::portfolio::Portfolio;
use crate::settings::Settings;
use anyhow::{Context, Result};
use std::rc::Rc;

pub fn show_holdings(name: &str, settings: Rc<Settings>) -> Result<()> {
    println!("Total PnL USD:");
    let pathbuf = path_from_name(name, settings).context("Failed to resolve portfolio path")?;
    Portfolio::print_unrealized_pnl(pathbuf)?;

    Ok(())
}
