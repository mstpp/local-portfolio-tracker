// #![allow(dead_code)]
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
    pub amount: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
}

/// Module to implment serde traits for inmported type OffsetDateTime
mod ts_seconds {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::OffsetDateTime;

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
        OffsetDateTime::from_unix_timestamp(ts).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

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
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for TradingPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "expected format 'BASE/QUOTE', got '{}'",
                s
            )));
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
}
