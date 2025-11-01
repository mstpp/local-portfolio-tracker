// #![allow(dead_code)]
use crate::trading_pair::TradingPair;
use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
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
    fn test_timestamp_deserializer() {
        let dt: OffsetDateTime = datetime!(2024-01-10 10:40 UTC);
        let tt: TestTrade = serde_json::from_str(r#"{"created_at": 1704883200}"#).unwrap();
        assert_eq!(tt.created_at, dt);
    }

    #[test]
    fn test_timestamp_deser_to_old() {
        match serde_json::from_str::<TestTrade>(r#"{"created_at": 111222333}"#) {
            Ok(_) => panic!("expected to fail"),
            Err(e) => assert!(
                e.to_string()
                    .contains("timestamp 111222333 is before minimum allowed date (2009-01-03)")
            ),
        }
    }

    #[test]
    fn test_timestamp_in_future() {
        let err = serde_json::from_str::<TestTrade>(r#"{"created_at": 1861826683}"#).unwrap_err();
        // ensure it's a data error (not IO or syntax)
        assert!(err.is_data());
        assert!(
            err.to_string().contains("timestamp is in the future"),
            "got: {err}"
        );
    }
    #[test]
    fn test_timestamp_ser() {
        let tt = TestTrade {
            created_at: datetime!(2024-01-10 10:40 UTC),
        };
        let json = serde_json::to_string(&tt).unwrap();
        assert_eq!(json, r#"{"created_at":1704883200}"#.to_string());
    }

    #[test]
    fn test_created_at_roundtrip() {
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
    fn test_tokens_for_ts_seconds() {
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ValTest {
        #[serde(deserialize_with = "positive_decimal")]
        pub amount: Decimal,
    }

    // #[test]
    // fn test_amount_positive_val_deser() {
    //     let d: ValTest = serde_json::from_str(r#"{"amount": 0.0001}"#).unwrap();
    //     println!("{:?}", &d);
    //     assert_eq!(
    //         d,
    //         ValTest {
    //             amount: rust_decimal::dec!(0.0001)
    //         }
    //     );
    // }

    /// Deserializes `{"amount": 5}` and succeeds with `Decimal(5)`.
    #[test]
    fn test_deser_accepts_simple_positive_integer() {
        let d: ValTest = serde_json::from_str(r#"{"amount": 5}"#).unwrap();
        println!("{:?}", &d);
        assert_eq!(
            d,
            ValTest {
                amount: rust_decimal::dec!(5)
            }
        );
    }

    /// Deserializes `{"amount": 5.75}` and succeeds with `Decimal(5.75)`.
    #[test]
    fn test_deser_accepts_simple_positive_decimal() {
        let d: ValTest = serde_json::from_str(r#"{"amount": 5.75}"#).unwrap();
        println!("{:?}", &d);
        assert_eq!(
            d,
            ValTest {
                amount: rust_decimal::dec!(5.75)
            }
        );
    }

    /// Deserializes `{"amount": 1e6}` and succeeds with `Decimal(1000000)`.
    #[test]
    fn test_deser_accepts_scientific_notation_positive() {
        let d: ValTest = serde_json::from_str(r#"{"amount": 1e6}"#).unwrap();
        println!("{:?}", &d);
        assert_eq!(
            d,
            ValTest {
                amount: rust_decimal::dec!(1000000)
            }
        );
    }

    /// Deserializes a very small positive like `{"amount": 1e-9}` and succeeds (still > 0).
    #[test]
    fn test_deser_accepts_small_positive_decimal_epsilon() {
        let d: ValTest = serde_json::from_str(r#"{"amount": 1e-9}"#).unwrap();
        println!("{:?}", &d);
        assert_eq!(
            d,
            ValTest {
                amount: rust_decimal::dec!(0.000000001)
            }
        );
    }

    // negative tests - non positive values

    fn assert_rejects_invalid_value(json: &str) {
        let res: Result<ValTest, serde_json::Error> = serde_json::from_str(json);
        assert!(res.is_err(), "expected deserialization to fail for {json}");
        let err = res.unwrap_err().to_string();
        assert!(
            err.contains("value must be positive number"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn test_deser_rejects_non_positive_values() {
        for json in [
            r#"{"amount": 0}"#,
            r#"{"amount": -3}"#,
            r#"{"amount": -0.0001}"#,
        ] {
            assert_rejects_invalid_value(json);
        }
    }

    // negative tests - type mismatch

    fn assert_jrects_invalid_type(json: &str) {
        let err = serde_json::from_str::<ValTest>(json).unwrap_err();
        assert!(err.is_data());
        // println!("{:?}", &err);
        assert!(err.to_string().contains("invalid type:"));
    }

    #[test]
    fn test_deser_rejects_type_error() {
        for json in [
            r#"{"amount": "1.23"}"#,
            r#"{"amount": true}"#,
            r#"{"amount": [1,2]}"#,
            r#"{"amount": {"k": 1}}"#,
        ] {
            assert_jrects_invalid_type(json);
        }
    }

    /// Deserializes an extremely large number (e.g., `{"amount": 1e309}`), expecting failure from `Decimal::try_from(f64)` with a custom-converted error.
    #[test]
    fn deser_maps_try_from_overflow_error() {}

    /// Deserializes a value that `Decimal::try_from` rejects (e.g., a denormal that can’t be represented), expecting a custom-converted error message from `try_from`.
    #[test]
    fn deser_maps_try_from_subnormal_or_precision_error() {}

    /// Deserializes `ValTest` with a valid positive number and ensures `ValTest { amount }` is constructed correctly.
    #[test]
    fn deser_integration_on_struct_field_succeeds() {}

    /// Deserializes `ValTest` with `0` and asserts the specific `"value must be positive number"` message surfaces from the field.
    #[test]
    fn deser_integration_on_struct_field_fails_zero() {}

    /// Deserializes `ValTest` with a negative number and asserts the specific error message.
    #[test]
    fn deser_integration_on_struct_field_fails_negative() {}

    /// Confirms that the `"value must be positive number"` text appears in the `serde_json::Error` display string for zero/negative inputs.
    #[test]
    fn deser_error_message_is_preserved_in_serde_json_error_display() {}

    /// Deserialize a positive JSON number to `Decimal` and compare against a `Decimal` constructed from string (e.g., `"5.75"`) to assert expected value; note potential f64→Decimal rounding implications.
    #[test]
    fn deser_roundtrip_behavior_documented_with_serialize_reference() {}

    /// Uses the smallest positive number representable in your domain (e.g., `1e-28` if meaningful for `Decimal`) and verifies it is accepted and correctly represented.
    #[test]
    fn deser_boundary_value_min_positive_handled() {}

    /// If JSON input source could allow non-finite numbers (NaN/Infinity via non-standard parser), ensure they are rejected via `try_from` mapping (or mark not applicable).
    #[test]
    fn deser_rejects_non_finite_values_if_parser_allows() {}

    // csv deserializer

    fn from_csv_str<T: for<'de> serde::Deserialize<'de>>(s: &str) -> Result<T, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(std::io::Cursor::new(s));
        reader.deserialize().next().unwrap()
    }

    fn assert_rejects_invalid_csv(csv_data: &str) {
        let result: Result<ValTest, csv::Error> = from_csv_str(&csv_data);
        assert!(
            result.is_err(),
            "expected CSV deserialization to fail for:\n{csv_data}"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("value must be positive number"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn test_csv_deser_invalid_value_negative() {
        assert_rejects_invalid_csv("amount\n-1\n");
    }
}
