use crate::settings::Settings;
use crate::trade::{Side, Trade, TradingPair};
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::rc::Rc;
use std::{fs, io::Write, path::PathBuf, time::SystemTime};

pub fn path_from_name(name: &str, settings: Rc<Settings>) -> Result<PathBuf> {
    Ok(settings
        .portfolio_dir
        .clone()
        .join(name)
        .with_extension("csv"))
}

fn is_csv_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

fn list(settings: Rc<Settings>) -> Result<Vec<String>> {
    let path_str = &settings.portfolio_dir.clone();
    let entries = fs::read_dir(path_str)
        .with_context(|| format!("Failed to read directory: {:?}", path_str))?;

    let mut csv_files: Vec<(String, SystemTime)> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_file() || !is_csv_file(&path) {
                return None;
            }

            let stem = path.file_stem()?.to_str()?.to_string();
            let created_at = entry.metadata().ok()?.created().ok()?;

            Some((stem, created_at))
        })
        .collect();

    // Sort by creation time
    csv_files.sort_by_key(|(_, time)| *time);

    Ok(csv_files.into_iter().map(|(name, _)| name).collect())
}

pub fn print_list(settings: Rc<Settings>) -> Result<()> {
    println!("\nFound csv files:\n");

    let l = list(settings)?;
    let size = l.len();
    if size == 0 {
        println!("no portfolios found");
    } else {
        println!("{}", l.join("\n"));
    }

    Ok(())
}

pub fn new(name: &str, settings: Rc<Settings>) -> Result<()> {
    let new_file_path: PathBuf = path_from_name(name, settings)?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&new_file_path)
        .with_context(|| format!("Error while creating csv file {:?}", &new_file_path))?;
    // write csv header
    file.write_all("created_at,pair,side,amount,price,fee\n".as_bytes())?;
    println!(
        "Successfully created new file: {}",
        new_file_path.to_str().unwrap_or("Unknown")
    );
    Ok(())
}

pub fn tx_to_csv(
    portfolio: &str,
    symbol: &str,
    side: &str,
    qty: Decimal,
    price: Decimal,
    fee: Decimal,
    settings: Rc<Settings>,
) -> Result<()> {
    let tx = Trade {
        created_at: time::OffsetDateTime::now_utc(),
        pair: serde_plain::from_str::<TradingPair>(&symbol).unwrap(),
        side: serde_plain::from_str::<Side>(&side).unwrap(),
        amount: qty,
        price: price,
        fee: fee,
    };

    let path = path_from_name(portfolio, settings)?;

    let csv_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect(format!("expecting csv file, but not found: {:?}", &path).as_str());
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(csv_file);
    wrt.serialize(&tx).unwrap();
    println!(
        "✅ Added transaction to portfolio csv file: {:?}\n{:?}",
        path, tx
    );
    Ok(())
}

pub fn read_trades_from_csv(name: &str, settings: Rc<Settings>) -> Result<Vec<Trade>> {
    let path = path_from_name(name, settings).context("Failed to resolve portfolio path")?;

    let file = std::fs::File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = csv::Reader::from_reader(file);

    // with filtering out lines that can't be deserialized 🪲 not a good idea, masking bugs (UTC timestamp)
    // let trades: Vec<Trade> = reader.deserialize().filter_map(Result::ok).collect();
    let trades: Vec<Trade> = reader
        .deserialize() // returns iterator of Result<Trade, csv::Error>
        .collect::<Result<Vec<Trade>, csv::Error>>()?;

    Ok(trades)
}

pub fn show_trades(name: &str, settings: Rc<Settings>) -> Result<()> {
    let trades = read_trades_from_csv(name, settings)?;
    if trades.len() == 0 {
        println!("No trades found in '{}'", &name);
    }
    for trade in trades {
        println!("{:?}", &trade);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures::{tickers, transactions};
    use crate::test_utils::helpers::transactions_from;
    use rstest::*;

    // not testing anything, more a showcase how to use fixtures
    #[rstest]
    fn test_read_trades_success(_tickers: (), transactions: Vec<Trade>) {
        let csv_content = r#"created_at,pair,side,amount,price,fee
1704883200,BTC/USD,BUY,1.0,40000.00,7.50
1710460800,BTC/USD,BUY,3,20000.00,10.00"#;
        let result = transactions_from(csv_content);
        assert_eq!(transactions, result);
    }

    // #[test]
    // fn test_read_trades_empty_file() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let csv_content = "symbol,quantity,price,date\n";

    //     create_test_csv(&temp_dir, "empty", csv_content);
    //     let settings = create_test_settings(temp_dir.path().to_path_buf());

    //     let result = read_trades_from_csv("empty", settings);

    //     assert!(result.is_ok());
    //     let trades = result.unwrap();
    //     assert_eq!(trades.len(), 0);
    // }

    //     #[test]
    //     fn test_read_trades_file_not_found() {
    //         let temp_dir = TempDir::new().unwrap();
    //         let settings = create_test_settings(temp_dir.path().to_path_buf());

    //         let result = read_trades_from_csv("nonexistent", settings);

    //         assert!(result.is_err());
    //     }

    //     #[test]
    //     fn test_read_trades_invalid_csv_format() {
    //         let temp_dir = TempDir::new().unwrap();
    //         let csv_content = r#"symbol,quantity,price,date
    // AAPL,not_a_number,150.50,2024-01-15"#;

    //         create_test_csv(&temp_dir, "invalid", csv_content);
    //         let settings = create_test_settings(temp_dir.path().to_path_buf());

    //         let result = read_trades_from_csv("invalid", settings);

    //         assert!(result.is_err());
    //     }

    //     #[test]
    //     fn test_read_trades_missing_columns() {
    //         let temp_dir = TempDir::new().unwrap();
    //         let csv_content = r#"symbol,quantity
    // AAPL,100"#;

    //         create_test_csv(&temp_dir, "missing_cols", csv_content);
    //         let settings = create_test_settings(temp_dir.path().to_path_buf());

    //         let result = read_trades_from_csv("missing_cols", settings);

    //         assert!(result.is_err());
    //     }

    //     #[test]
    //     fn test_read_trades_with_special_characters() {
    //         let temp_dir = TempDir::new().unwrap();
    //         let csv_content = r#"symbol,quantity,price,date
    // "AAPL, Inc",100,150.50,2024-01-15"#;

    //         create_test_csv(&temp_dir, "special", csv_content);
    //         let settings = create_test_settings(temp_dir.path().to_path_buf());

    //         let result = read_trades_from_csv("special", settings);

    //         // Should handle CSV escaping properly
    //         assert!(result.is_ok());
    //     }

    //     #[test]
    //     fn test_read_trades_large_file() {
    //         let temp_dir = TempDir::new().unwrap();
    //         let mut csv_content = String::from("symbol,quantity,price,date\n");

    //         // Generate 1000 trades
    //         for i in 0..1000 {
    //             csv_content.push_str(&format!("SYM{},100,150.50,2024-01-15\n", i));
    //         }

    //         create_test_csv(&temp_dir, "large", &csv_content);
    //         let settings = create_test_settings(temp_dir.path().to_path_buf());

    //         let result = read_trades_from_csv("large", settings);

    //         assert!(result.is_ok());
    //         let trades = result.unwrap();
    //         assert_eq!(trades.len(), 1000);
    //     }
}
