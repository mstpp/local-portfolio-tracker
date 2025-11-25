use serde::{Deserialize, Serialize};
use std::fmt;

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
