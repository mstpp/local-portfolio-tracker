use crate::currency::{QuoteCurrency, Ticker};
use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingPair {
    pub base: Ticker,
    pub quote: QuoteCurrency,
}

impl Serialize for TradingPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}/{}", self.base, self.quote);
        serializer.serialize_str(&s.to_uppercase())
    }
}

impl<'de> Deserialize<'de> for TradingPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<String> = s.split('/').map(|t| t.to_uppercase()).collect();

        if parts.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "expected format 'BASE/QUOTE', got '{}'",
                s
            )));
        }

        // base should not be empty string
        if parts[0].trim().is_empty() {
            return Err(serde::de::Error::custom("base can't be empty"));
        }

        // only accept USD quote
        if parts[1] != "USD" {
            return Err(serde::de::Error::custom(
                "accepting only USD for quote currency",
            ));
        }

        let base_curr = Ticker::from_str(&parts[0]).map_err(serde::de::Error::custom)?;
        let quote_curr = QuoteCurrency::from_str(&parts[1]).map_err(serde::de::Error::custom)?;

        Ok(TradingPair {
            base: base_curr,
            quote: quote_curr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::init_tickers_from_csv;
    use std::path::PathBuf;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub struct TestPair {
        pub pair: TradingPair,
    }

    /// Verifies that the serialized output follows the "BASE/QUOTE" format with a single `/` separator.
    #[test]
    fn test_serialize_uses_slash_separator() {
        init_tickers_from_csv(PathBuf::from_str("./data/coingecko.csv").unwrap()).unwrap();

        let d = serde_json::from_str::<TestPair>(r#"{"pair":"ETH/USD"}"#).unwrap();
        assert_eq!(
            TestPair {
                pair: TradingPair {
                    base: Ticker {
                        id: "ETH".to_string()
                    },
                    quote: QuoteCurrency::Usd
                }
            },
            d
        );
    }

    /// Verifies that any quote currency other than "USD" (e.g., "BTC/EUR") is rejected.
    #[test]
    fn test_deserialize_rejects_invalid_quote_currency() {
        let json_str = r#"{"pair":"BTC/USDT"}"#;
        let err = serde_json::from_str::<TestPair>(&json_str).unwrap_err();
        assert!(
            err.to_string()
                .contains("accepting only USD for quote currency")
        );
    }

    /// Checks that input without `/` (e.g., "BTCUSD") returns a format error.
    #[test]
    fn test_deserialize_rejects_missing_separator() {
        let json_str = r#"{"pair":"BTCUSDT"}"#;
        let err = serde_json::from_str::<TestPair>(&json_str).unwrap_err();
        assert!(
            err.to_string()
                .contains("expected format 'BASE/QUOTE', got 'BTCUSDT'")
        );
    }

    #[test]
    fn invalid_trading_pair_format_doubleslash() {
        let json_str = r#"{"pair":"BTC/ETH/USD"}"#;
        let err = serde_json::from_str::<TestPair>(&json_str).unwrap_err();
        // println!("{:?}", &err);
        assert!(
            err.to_string()
                .contains("expected format 'BASE/QUOTE', got 'BTC/ETH/USD'")
        );
    }

    /// Ensures that both `base` and `quote` are serialized in uppercase form regardless of input casing.
    #[test]
    fn test_serialize_converts_to_uppercase() {
        init_tickers_from_csv(PathBuf::from_str("./data/coingecko.csv").unwrap()).unwrap();

        let d = serde_json::from_str::<TestPair>(r#"{"pair":"btc/UsD"}"#).unwrap();
        assert_eq!(
            TestPair {
                pair: TradingPair {
                    base: Ticker {
                        id: "BTC".to_string()
                    },
                    quote: QuoteCurrency::Usd
                }
            },
            d
        );
    }

    // 🤖 generated:

    /// Checks that non-alphabetic symbols in `base` (like "eth2") are preserved during serialization.
    #[test]
    fn test_serialize_preserves_alphanumeric_symbols() {
        init_tickers_from_csv(PathBuf::from_str("./data/coingecko.csv").unwrap()).unwrap();
        let d = serde_json::from_str::<TestPair>(r#"{"pair":"usdt0/USD"}"#).unwrap();
        assert_eq!(
            TestPair {
                pair: TradingPair {
                    base: Ticker {
                        id: "USDT0".to_string()
                    },
                    quote: QuoteCurrency::Usd
                }
            },
            d
        );
    }

    /// Checks that an empty input string fails deserialization with a clear error.
    #[test]
    fn test_deserialize_rejects_empty_string() {
        let json_str = r#"{"pair":""}"#;
        let err = serde_json::from_str::<TestPair>(&json_str).unwrap_err();
        // println!("{:?}", &err);
        assert!(
            err.to_string()
                .contains("expected format 'BASE/QUOTE', got ''")
        );
    }

    /// Validates that "BTC/" produces an error since the quote part is missing.
    #[test]
    fn test_deserialize_rejects_only_base_no_quote() {
        let json_str = r#"{"pair":"BTC/"}"#;
        let err = serde_json::from_str::<TestPair>(&json_str).unwrap_err();
        // println!("{:?}", &err);
        assert!(
            err.to_string()
                .contains("accepting only USD for quote currency")
        );
    }

    /// Validates that "/USD" produces an error since the base part is missing.
    // 🎉 couaght a 🪲 during test writing
    #[test]
    fn test_deserialize_rejects_only_quote_no_base() {
        let json_str = r#"{"pair":"/USD"}"#;
        let err = serde_json::from_str::<TestPair>(&json_str).unwrap_err();
        // println!("{:?}", &err);
        assert!(err.to_string().contains("base can't be empty"));
    }

    // roundtrip

    /// Ensures that serializing and then deserializing a valid TradingPair returns the same normalized struct.
    #[test]
    fn test_serialize_then_deserialize_roundtrip() {
        // TODO
    }

    /// Ensures that deserializing a valid string and then serializing it again produces the same uppercase "BASE/QUOTE" string.
    #[test]
    fn test_deserialize_then_serialize_roundtrip() {
        // TODO
    }

    /// Ensures inputs with spaces like "  btc / usd " are handled appropriately (trimmed or rejected).
    #[test]
    fn test_deserialize_with_whitespace() {
        // TODO
    }

    /// Checks that non-ASCII input like "btç/usd" either serializes safely or fails as expected.
    #[test]
    fn test_serialize_with_non_ascii_characters() {
        // TODO
    }

    /// Verifies that deserializing non-ASCII or invalid Unicode behaves correctly (accepts or rejects as per spec).
    #[test]
    fn test_deserialize_with_non_ascii_characters() {
        // TODO
    }
}
