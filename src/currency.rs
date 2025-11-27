use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

static TICKERS: OnceLock<HashSet<Box<String>>> = OnceLock::new();

pub fn init_tickers_from_csv(path: PathBuf) -> Result<()> {
    if TICKERS.get().is_none() {
        let file = std::fs::File::open(path)?;
        let mut rd = csv::Reader::from_reader(file);
        let mut set: HashSet<Box<String>> = HashSet::new();
        let _: Vec<_> = rd
            .deserialize::<(String, String, String)>()
            .map(|res| {
                if let Ok(r) = res {
                    let ticker = r.1.to_ascii_uppercase();
                    set.insert(Box::new(ticker));
                }
            })
            .collect();

        // many duplicate tickers even if first 100 coins
        let _ = TICKERS.set(set); // ignoring error if try to add duplicate 
        // TODO fix tests
        // .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Ticker init err "));
    }

    Ok(())
}

pub fn is_valid_ticker(t: &str) -> bool {
    let key = t.trim().to_ascii_uppercase();
    TICKERS
        .get()
        .expect("TICKERS not initialized; call init_tickers_from_csv() first")
        .contains(&key)
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
        if is_valid_ticker(&s) {
            Ok(Self { id: s })
        } else {
            Err(serde::de::Error::custom(format!("Invalid ticker {}", s)))
        }
    }
}

impl fmt::Display for Ticker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id.to_ascii_uppercase())
    }
}

impl std::str::FromStr for Ticker {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_valid_ticker(&s) {
            Ok(Self {
                id: s.to_ascii_uppercase(),
            })
        } else {
            Err(format!("Invalid ticker {}", s))
        }
    }
}

/// Represents the base currency for portfolio valuation
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
