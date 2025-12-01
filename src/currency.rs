use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

pub static TICKERS: OnceLock<HashSet<String>> = OnceLock::new();

#[derive(Deserialize)]
struct CsvRow {
    #[allow(dead_code)]
    id: String,
    #[serde(rename = "symbol")]
    ticker: String,
    #[allow(dead_code)]
    name: String,
}

fn normalize_ticker(s: &str) -> String {
    s.trim().to_ascii_uppercase()
}

pub fn load_tickers_from_csv(path: PathBuf) -> Result<HashSet<String>> {
    let file = std::fs::File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut tickers = HashSet::new();

    for result in reader.deserialize::<CsvRow>() {
        match result {
            Ok(row) => {
                tickers.insert(normalize_ticker(&row.ticker));
            }
            Err(e) => {
                eprintln!("Failed to parse CSV row: {e}");
            }
        }
    }

    Ok(tickers)
}

fn tickers() -> &'static HashSet<String> {
    TICKERS
        .get()
        .expect("TICKERS not initialized; call init_tickers_from_csv() first")
}

pub fn is_valid_ticker(t: &str) -> bool {
    tickers().contains(&normalize_ticker(t))
}

pub fn init_tickers_from_csv(path: PathBuf) -> Result<()> {
    if TICKERS.get().is_some() {
        return Ok(());
    }

    let tickers = load_tickers_from_csv(path)?;

    if TICKERS.set(tickers).is_err() {
        println!("TICKERS already initialized, ignoring later init");
    }

    Ok(())
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Ticker {
    pub id: String,
}

impl<'de> Deserialize<'de> for Ticker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let normalized = normalize_ticker(&s);

        if is_valid_ticker(&normalized) {
            Ok(Self { id: normalized })
        } else {
            Err(serde::de::Error::custom(format!("Invalid ticker: {}", s)))
        }
    }
}

impl fmt::Display for Ticker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl std::str::FromStr for Ticker {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_ticker(s);

        if is_valid_ticker(&normalized) {
            Ok(Self { id: normalized })
        } else {
            Err(format!("Invalid ticker: {}", s))
        }
    }
}

// QuoteCurrency implementation looks good, no changes needed
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteCurrency {
    Usd,
    Btc,
}

impl fmt::Display for QuoteCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usd => write!(f, "USD"),
            Self::Btc => write!(f, "BTC"),
        }
    }
}

impl std::str::FromStr for QuoteCurrency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Self::Usd),
            "BTC" => Ok(Self::Btc),
            _ => Err(format!("Invalid currency: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures::tickers;
    use rstest::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    mod basic {
        use std::str::FromStr;

        use super::*;

        #[rstest]
        fn test_from_str(_tickers: ()) {
            assert_eq!(
                Ticker::from_str("btc"),
                Ok(Ticker {
                    id: "BTC".to_string()
                })
            )
        }

        #[test]
        fn test_normalize_ticker() {
            assert_eq!(&normalize_ticker("ABC"), "ABC");
            assert_eq!(&normalize_ticker(" AbC "), "ABC");
            assert_eq!(&normalize_ticker("\n abc\t "), "ABC");
        }

        // #[test]
        // fn test_load_ticker_from_csv() {
        //     let res = setup_tickers().unwrap();
        //     assert!(res.contains("BTC"));
        //     assert!(res.contains("ETH"));
        //     assert!(res.contains("USDT"));
        //     assert!(res.contains("BNB"));
        //     assert!(res.contains("ADA"));
        //     assert!(res.contains("USDT0"));
        // }

        // #[test]
        // fn test_idempotent_init() {
        //     init_tickers();
        //     init_tickers();
        //     assert!(is_valid_ticker("BTC"));
        // }

        #[rstest]
        fn test_valid_ticker(_tickers: ()) {
            assert!(is_valid_ticker("BTC"));
            assert!(is_valid_ticker("ADA"));
            assert!(is_valid_ticker("eth "));
            assert!(is_valid_ticker("  eth "));
            assert!(is_valid_ticker(" \t eth  \n "));
        }

        #[rstest]
        fn test_invalid_ticker(_tickers: ()) {
            assert!(!is_valid_ticker("abtc"));
            assert!(!is_valid_ticker("aaADA"));
            assert!(!is_valid_ticker(""));
            assert!(!is_valid_ticker("\t"));
        }

        #[test]
        #[should_panic(expected = "TICKERS not initialized")]
        fn test_panic_when_no_init() {
            is_valid_ticker("abtc");
        }

        #[rstest]
        fn test_ticker_from_str_invalid(_tickers: ()) {
            let result: Result<Ticker, _> = "INVALID".parse();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Invalid ticker"));
        }

        #[rstest]
        fn test_ticker_from_str_normalizes(_tickers: ()) {
            let ticker1: Ticker = "btc".parse().unwrap();
            let ticker2: Ticker = "BTC".parse().unwrap();
            let ticker3: Ticker = "  btc  ".parse().unwrap();

            assert_eq!(ticker1.id, "BTC");
            assert_eq!(ticker2.id, "BTC");
            assert_eq!(ticker3.id, "BTC");
        }

        #[rstest]
        fn test_ticker_display(_tickers: ()) {
            let ticker: Ticker = "btc".parse().unwrap();
            assert_eq!(ticker.to_string(), "BTC");
        }

        #[rstest]
        fn test_ticker_serialize(_tickers: ()) {
            let ticker: Ticker = "btc".parse().unwrap();
            let json = serde_json::to_string(&ticker).unwrap();
            assert_eq!(json, r#"{"id":"BTC"}"#);
        }
    }

    mod quote_currency {
        use super::*;

        #[test]
        fn test_quote_currency_from_str_usd() {
            let currency: QuoteCurrency = "USD".parse().unwrap();
            assert_eq!(currency, QuoteCurrency::Usd);
        }

        #[test]
        fn test_quote_currency_from_str_btc() {
            let currency: QuoteCurrency = "BTC".parse().unwrap();
            assert_eq!(currency, QuoteCurrency::Btc);
        }

        #[test]
        fn test_quote_currency_from_str_case_insensitive() {
            assert_eq!("usd".parse::<QuoteCurrency>().unwrap(), QuoteCurrency::Usd);
            assert_eq!("Usd".parse::<QuoteCurrency>().unwrap(), QuoteCurrency::Usd);
            assert_eq!("btc".parse::<QuoteCurrency>().unwrap(), QuoteCurrency::Btc);
            assert_eq!("BtC".parse::<QuoteCurrency>().unwrap(), QuoteCurrency::Btc);
        }

        #[test]
        fn test_quote_currency_from_str_invalid() {
            let result = "EUR".parse::<QuoteCurrency>();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Invalid currency"));
        }

        #[test]
        fn test_quote_currency_display() {
            assert_eq!(QuoteCurrency::Usd.to_string(), "USD");
            assert_eq!(QuoteCurrency::Btc.to_string(), "BTC");
        }

        #[test]
        fn test_quote_currency_serialize() {
            let usd_json = serde_json::to_string(&QuoteCurrency::Usd).unwrap();
            let btc_json = serde_json::to_string(&QuoteCurrency::Btc).unwrap();

            assert_eq!(usd_json, r#""Usd""#);
            assert_eq!(btc_json, r#""Btc""#);
        }

        #[test]
        fn test_quote_currency_deserialize() {
            let usd: QuoteCurrency = serde_json::from_str(r#""Usd""#).unwrap();
            let btc: QuoteCurrency = serde_json::from_str(r#""Btc""#).unwrap();

            assert_eq!(usd, QuoteCurrency::Usd);
            assert_eq!(btc, QuoteCurrency::Btc);
        }

        #[test]
        fn test_quote_currency_equality() {
            assert_eq!(QuoteCurrency::Usd, QuoteCurrency::Usd);
            assert_eq!(QuoteCurrency::Btc, QuoteCurrency::Btc);
            assert_ne!(QuoteCurrency::Usd, QuoteCurrency::Btc);
        }

        #[test]
        fn test_quote_currency_copy() {
            let currency1 = QuoteCurrency::Usd;
            let currency2 = currency1; // Copy, not move
            assert_eq!(currency1, currency2);
        }

        #[test]
        fn test_quote_currency_hash() {
            let mut set = HashSet::new();
            set.insert(QuoteCurrency::Usd);
            set.insert(QuoteCurrency::Btc);
            set.insert(QuoteCurrency::Usd); // Duplicate

            assert_eq!(set.len(), 2);
        }
    }

    mod edge_cases {
        use super::*;

        #[rstest]
        fn test_empty_ticker_string(_tickers: ()) {
            let result: Result<Ticker, _> = "".parse();
            assert!(
                result
                    .expect_err("Expected err")
                    .contains("Invalid ticker:")
            )
        }

        #[rstest]
        fn test_whitespace_only_ticker(_tickers: ()) {
            let result: Result<Ticker, _> = "   ".parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_csv_with_malformed_rows() {
            let mut file = NamedTempFile::new().unwrap();
            writeln!(
                file,
                "id,symbol,name\n\
                 bitcoin,btc,Bitcoin\n\
                 incomplete,line\n\
                 ethereum,eth,Ethereum"
            )
            .unwrap();
            file.flush().unwrap();

            // Should still succeed and parse valid rows
            let result = init_tickers_from_csv(file.path().to_path_buf());
            assert!(result.is_ok());
        }

        #[test]
        fn test_csv_with_duplicate_tickers() {
            let mut file = NamedTempFile::new().unwrap();
            writeln!(
                file,
                "id,symbol,name\n\
                 bitcoin,btc,Bitcoin\n\
                 bitcoin2,btc,Bitcoin Cash"
            )
            .unwrap();
            file.flush().unwrap();

            let result = init_tickers_from_csv(file.path().to_path_buf());
            assert!(result.is_ok());

            // Should only have one BTC entry
            assert!(is_valid_ticker("BTC"));
        }
    }
}
