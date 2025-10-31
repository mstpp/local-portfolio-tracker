// #![allow(dead_code)]
use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::OffsetDateTime;

/// Represents a single executed trade in a portfolio.
///
/// This struct is designed to be serialized and deserialized to/from CSV
///
/// Example
///
/// Example of one trade entry in CSV file:
/// ```csv
/// created_at,pair,side,amount,price,fee
/// 1704883200,BTC/USD,BUY,1.0,40000.00,7.50
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    /// In the csv file we prefer to have epoch as timestamp,
    /// while in the runtime we would like to have OffsetDateTime type
    #[serde(with = "ts_seconds")]
    pub created_at: OffsetDateTime,
    pub pair: TradingPair,
    pub side: Side,
    #[serde(deserialize_with = "positive_decimal")]
    pub amount: Decimal,
    #[serde(deserialize_with = "positive_decimal")]
    pub price: Decimal,
    #[serde(deserialize_with = "positive_decimal")] // TODO accept fee=0.0
    pub fee: Decimal,
}

// Learning 📖
// ⛔️ Not using validation fn, it needs to be called explicitly
// better approach: validate as part of deserialization
// impl Trade {
//     pub fn validate(&self) -> Result<()> {
//         if self.amount <= Decimal::ZERO {
//             return Err(anyhow::anyhow!("value must be > 0"));
//         }
//         if self.price <= Decimal::ZERO {
//             return Err(anyhow::anyhow!("price must be > 0"));
//         }
//         if self.fee < Decimal::ZERO {
//             return Err(anyhow::anyhow!("negative fee not allowed"));
//         }
//         Ok(())
//     }
// }

fn positive_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let num = f64::deserialize(deserializer)?;
    let d = Decimal::try_from(num).map_err(serde::de::Error::custom)?;
    if d <= Decimal::ZERO {
        return Err(serde::de::Error::custom("value must be positive number"));
    }
    Ok(d)
}

/// Module to implment serde traits for inmported type OffsetDateTime
mod ts_seconds {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::OffsetDateTime;

    // January 3, 2009 at 00:00:00 UTC (Bitcoin genesis block date)
    const MIN_TIMESTAMP: i64 = 1231027200;

    pub fn serialize<S>(dt: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dt.unix_timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts = i64::deserialize(deserializer)?;

        // Validate: check minimum date (January 3, 2009)
        if ts < self::MIN_TIMESTAMP {
            return Err(serde::de::Error::custom(format!(
                "timestamp {} is before minimum allowed date (2009-01-03)",
                ts
            )));
        }

        // Validate: timestamp can't be in the future
        let now_epoch = OffsetDateTime::now_utc().unix_timestamp();
        if ts >= now_epoch {
            return Err(serde::de::Error::custom("timestamp is in the future"));
        }

        // miliseconds are implicitly rejected
        OffsetDateTime::from_unix_timestamp(ts).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

// Learning 📖
// Example of manually implementing ser/deser traits
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingPair {
    pub base: String,
    pub quote: String,
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

        // only accept USD as quote for now
        if parts[1] != "USD" {
            return Err(serde::de::Error::custom(
                "accepting only USD for quote currency",
            ));
        }

        Ok(TradingPair {
            base: parts[0].to_string(),
            quote: parts[1].to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use time::macros::datetime;

    // - - - - - - - - - - - - - - - - - - - - - - - -
    // ts_seconds module test
    // - - - - - - - - - - - - - - - - - - - - - - - -

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub struct TestTrade {
        #[serde(with = "ts_seconds")]
        pub created_at: OffsetDateTime,
    }

    #[test]
    fn timestamp_deser() {
        let json = r#"{"created_at": 1704883200}"#;
        let dt: OffsetDateTime = datetime!(2024-01-10 10:40 UTC);
        let tt: TestTrade = serde_json::from_str(json).unwrap();
        assert_eq!(tt.created_at, dt);
    }

    #[test]
    fn timestamp_deser_to_old() {
        let json = r#"{"created_at": 111222333}"#;
        match serde_json::from_str::<TestTrade>(json) {
            Ok(_) => panic!("expected to fail"),
            Err(e) => assert!(
                e.to_string()
                    .contains("timestamp 111222333 is before minimum allowed date (2009-01-03)")
            ),
        }
    }

    #[test]
    fn timestamp_in_future() {
        let json = r#"{"created_at": 1861826683}"#;
        let err = serde_json::from_str::<TestTrade>(json).unwrap_err();
        // ensure it's a data error (not IO or syntax)
        assert!(err.is_data());
        assert!(
            err.to_string().contains("timestamp is in the future"),
            "got: {err}"
        );
    }
    #[test]
    fn timestamp_ser() {
        let tt = TestTrade {
            created_at: datetime!(2024-01-10 10:40 UTC),
        };
        let json = serde_json::to_string(&tt).unwrap();
        assert_eq!(json, r#"{"created_at":1704883200}"#.to_string());
    }

    #[test]
    fn round_trip() {
        let original = TestTrade {
            created_at: datetime!(2024-01-10 10:40 UTC),
        };
        let json_val = json!({"created_at": 1704883200});
        // serialize test
        let ser_val = serde_json::to_value(&original).unwrap();
        assert_eq!(ser_val, json_val);
        // deserialize test
        let deser_val: TestTrade = serde_json::from_value(json_val).unwrap();
        assert_eq!(deser_val.created_at, original.created_at);
    }

    #[test]
    fn tokens_for_ts_seconds() {
        use serde_test::{Token, assert_tokens};
        let t = TestTrade {
            created_at: datetime!(2024-01-10 10:40 UTC),
        };

        assert_tokens(
            &t,
            &[
                Token::Struct {
                    name: "TestTrade",
                    len: 1,
                },
                Token::Str("created_at"),
                Token::I64(1704883200),
                Token::StructEnd,
            ],
        );
    }

    // - - - - - - - - - - - - - - - - - - - - - - - -

    // - - - - - - - - - - - - - - - - - - - - - - - -
    // positive_decimal deserializer test
    // - - - - - - - - - - - - - - - - - - - - - - - -

    #[derive(Debug, Serialize, Deserialize)]
    struct ValTest {
        #[serde(deserialize_with = "positive_decimal")]
        pub amount: Decimal,
    }

    #[test]
    fn positive_val_deser() {
        let json = r#"{"amount": 0.0001}"#;
        let d: ValTest = serde_json::from_str(&json).unwrap();
        println!("{:?}", &d);
        // TODO
    }

    // TODO boundary tests
    // TOOD negative value

    // - - - - - - - - - - - - - - - - - - - - - - - -
    // TradingPair ser/deser test
    // - - - - - - - - - - - - - - - - - - - - - - - -

    /// Verifies that the serialized output follows the "BASE/QUOTE" format with a single `/` separator.
    #[test]
    fn test_serialize_uses_slash_separator() {
        let d = serde_json::from_str::<TestPair>(r#"{"pair":"ETH/USD"}"#).unwrap();
        assert_eq!(
            TestPair {
                pair: TradingPair {
                    base: "ETH".to_string(),
                    quote: "USD".to_string()
                }
            },
            d
        );
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub struct TestPair {
        pub pair: TradingPair,
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
        println!("{:?}", &err);
        assert!(
            err.to_string()
                .contains("expected format 'BASE/QUOTE', got 'BTC/ETH/USD'")
        );
    }

    /// Ensures that both `base` and `quote` are serialized in uppercase form regardless of input casing.
    #[test]
    fn test_serialize_converts_to_uppercase() {
        let d = serde_json::from_str::<TestPair>(r#"{"pair":"btc/UsD"}"#).unwrap();
        assert_eq!(
            TestPair {
                pair: TradingPair {
                    base: "BTC".to_string(),
                    quote: "USD".to_string()
                }
            },
            d
        );
    }

    // 🤖 generated:

    /// Checks that non-alphabetic symbols in `base` (like "eth2") are preserved during serialization.
    #[test]
    fn test_serialize_preserves_alphanumeric_symbols() {
        // TODO
    }

    /// Checks that an empty input string fails deserialization with a clear error.
    #[test]
    fn test_deserialize_rejects_empty_string() {
        // TODO
    }

    /// Validates that "BTC/" produces an error since the quote part is missing.
    #[test]
    fn test_deserialize_rejects_only_base_no_quote() {
        // TODO
    }

    /// Validates that "/USD" produces an error since the base part is missing.
    #[test]
    fn test_deserialize_rejects_only_quote_no_base() {
        // TODO
    }

    /// Ensures that serializing and then deserializing a valid TradingPair returns the same normalized struct.
    #[test]
    fn test_serialize_then_deserialize_roundtrip() {
        // TODO
    }

    // roundtrip
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
