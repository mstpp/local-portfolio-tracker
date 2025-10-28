// #![allow(dead_code)]
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    #[serde(with = "ts_seconds")]
    pub created_at: OffsetDateTime,
    pub pair: TradingPair,
    pub side: Side,
    pub amount: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
}

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
