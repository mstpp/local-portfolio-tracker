use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::sync::LazyLock;

pub const FIAT: &[&str] = &["USD", "EUR", "CAD"];
pub const STABLE: &[&str] = &["USDC", "USDT", "USDS", "DAI", "USDE"];
pub static CRYPTO: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    HashSet::from([
        "BTC", "ETH", "XRP", "BNB", "SOL", "TRX", "DOGE", "ADA", "BCH", "LINK", "HYPE", "LEO",
        "WETH", "XLM", "XMR", "SUI", "AVAX", "LTC", "HBAR", "ZEC", "SHIB", "CRO", "TON", "DOT",
        "UNI", "MNT", "AAVE", "TAO", "BGB", "M", "S", "OKB", "NEAR", "ASTER", "ETC", "ICP", "PI",
        "PEPE", "RAIN", "PUMP", "ONDO", "HTX", "JLP", "KAS",
    ])
});

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Currency {
    pub ticker: String,
    pub currency_type: CurrencyType,
}

impl Default for Currency {
    fn default() -> Self {
        Self {
            ticker: "USD".to_string(),
            currency_type: CurrencyType::Fiat,
        }
    }
}
impl Currency {
    /// USD is the base currency
    pub fn is_usd(&self) -> bool {
        self.ticker.as_str() == "USD"
    }

    pub fn from_ticker(s: &str) -> Result<Self> {
        Ok(Currency {
            ticker: normalize_ticker(s),
            // TODO rm implicit ticker validation
            currency_type: CurrencyType::from_ticker(s)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    Crypto,
    Fiat,
    StableCoin,
}

impl CurrencyType {
    pub fn from_ticker(s: &str) -> Result<Self> {
        let t = normalize_ticker(s);
        if FIAT.contains(&t.as_str()) {
            return Ok(CurrencyType::Fiat);
        } else if STABLE.contains(&t.as_str()) {
            return Ok(CurrencyType::StableCoin);
        } else if CRYPTO.contains(&t.as_str()) {
            return Ok(CurrencyType::Crypto);
        } else {
            Err(anyhow!("unsuported ticker {}", t))
        }
    }
}

pub fn is_valid_ticker(t: &str) -> bool {
    CRYPTO.contains(normalize_ticker(t).as_str()) || FIAT.contains(&t) || STABLE.contains(&t)
}

fn normalize_ticker(s: &str) -> String {
    s.trim().to_ascii_uppercase()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures::tickers;
    use rstest::*;

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
    }
}
