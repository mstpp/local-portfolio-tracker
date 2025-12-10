use crate::currency::Currency;
use crate::currency::{QuoteCurrency, Ticker};
use crate::settings::Settings;
use crate::tx::Tx;
use anyhow::{Context, Result};
use prettytable::{Cell, Row, Table, row};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use time::{OffsetDateTime, format_description};

// TODO could this be replaced with serialized Trade?
static CSV_HEADER: [&str; 6] = ["created_at", "pair", "side", "amount", "price", "fee"];

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
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
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

impl Trade {
    pub fn to_tx(&self) -> Result<Tx> {
        match self.side {
            Side::Buy => Ok(Tx {
                buy: Currency::from_ticker(&self.pair.base.id)?,
                buy_size: self.amount,
                sell: Currency::from_ticker(&self.pair.quote.to_string())?,
                sell_size: self.amount * self.price + self.fee,
            }),
            Side::Sell => Ok(Tx {
                buy: Currency::from_ticker(&self.pair.quote.to_string())?,
                buy_size: self.amount * self.price - self.fee,
                sell: Currency::from_ticker(&self.pair.base.id)?,
                sell_size: self.amount,
            }),
        }
    }

    pub fn to_table_row(&self) -> Row {
        let datetime = self
            .created_at
            .format(&format_description::well_known::Rfc2822)
            .unwrap_or_else(|_| "Invalid date".to_string());
        row![
            datetime,
            self.pair,
            self.side,
            self.amount,
            self.price,
            self.fee
        ]
    }
}

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
        if ts > now_epoch {
            return Err(serde::de::Error::custom(
                format!(
                    "timestamp is in the future: {:?}\ncurrent timestamp: {now_epoch}",
                    &ts
                )
                .as_str(),
            ));
        }

        // miliseconds are implicitly rejected
        OffsetDateTime::from_unix_timestamp(ts).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

/// Accepting any case, but serialize to uppercase
impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a plain string first
        let s = String::deserialize(deserializer)?;
        match s.trim().to_ascii_uppercase().as_str() {
            "BUY" => Ok(Side::Buy),
            "SELL" => Ok(Side::Sell),
            other => Err(serde::de::Error::unknown_variant(other, &["BUY", "SELL"])),
        }
    }
}

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

        // // only accept USD quote
        // if parts[1] != "USD" {
        //     return Err(serde::de::Error::custom(
        //         "accepting only USD for quote currency",
        //     ));
        // }

        let base_curr = Ticker::from_str(&parts[0]).map_err(serde::de::Error::custom)?;
        let quote_curr = QuoteCurrency::from_str(&parts[1]).map_err(serde::de::Error::custom)?;

        Ok(TradingPair {
            base: base_curr,
            quote: quote_curr,
        })
    }
}

impl fmt::Display for TradingPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
        }
    }
}
/// Create a new trades CSV file with headers
pub fn new(name: &str, settings: &Settings) -> Result<()> {
    let file_path = settings.path_for(name);

    let mut wtr = csv::Writer::from_path(&file_path)
        .with_context(|| format!("Failed to create trades CSV at {:?}", &file_path))?;

    // Explicitly write header
    wtr.write_record(CSV_HEADER)?;
    wtr.flush()?;

    println!("Created trades file: {}", file_path.display());
    Ok(())
}

/// Add new tx to csv portfolio file
pub fn tx_to_csv(
    portfolio: &str,
    symbol: &str,
    side: &str,
    qty: Decimal,
    price: Decimal,
    fee: Decimal,
    settings: &Settings,
) -> Result<()> {
    let tx = Trade {
        created_at: time::OffsetDateTime::now_utc(),
        pair: serde_plain::from_str::<TradingPair>(&symbol).unwrap(),
        side: serde_plain::from_str::<Side>(&side).unwrap(),
        amount: qty,
        price: price,
        fee: fee,
    };

    let path = settings.path_for(portfolio);

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

pub fn read_trades_from_csv(name: &str, settings: &Settings) -> Result<Vec<Trade>> {
    let path = settings.path_for(name);
    let file = std::fs::File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = csv::Reader::from_reader(file);
    let trades: Vec<Trade> = reader
        .deserialize() // returns iterator of Result<Trade, csv::Error>
        .collect::<Result<Vec<Trade>, csv::Error>>()?;
    Ok(trades)
}

pub fn show_trades(name: &str, settings: &Settings) -> Result<()> {
    let path = settings.path_for(name);
    let file = std::fs::File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = csv::Reader::from_reader(file);

    // prettytable
    let mut table = Table::new();
    let header_row = Row::new(CSV_HEADER.iter().map(|&c| Cell::new(c)).collect());
    table.add_row(header_row);

    for res in reader.deserialize() {
        let t: Trade = res?;
        let row = t.to_table_row();
        table.add_row(row);
    }

    table.printstd();

    Ok(())
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

    mod trading_pair {
        use super::*;
        use crate::test_utils::fixtures::tickers;
        use rstest::*;

        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        pub struct TestPair {
            pub pair: TradingPair,
        }

        // validate base and quote can't be the same
        #[rstest]
        fn test_base_and_quote_cannot_be_the_same_usd(_tickers: ()) {
            let p = serde_json::from_str::<TestPair>(r#"{"pair":"USD/USD"}"#);
            // Invalid ticker: USD
            assert!(p.is_err(), "expected err, got {:?}", &p);
        }

        #[rstest]
        fn test_base_and_quote_cannot_be_the_same_btc(_tickers: ()) {
            let p = serde_json::from_str::<TestPair>(r#"{"pair":"BTC/BTC"}"#);
            // TODO once we accept both btc and usd, this shoud not fail
            // currently it's passing since only USD is allowed
            println!("{:?}", &p);
            assert!(p.is_err(), "expected err, got {:?}", &p);

            // let pair = TradingPair {
            //     base: Ticker::from_str("BTC").unwrap(),
            //     quote: QuoteCurrency::Btc,
            // };
        }

        /// Verifies that the serialized output follows the "BASE/QUOTE" format with a single `/` separator.
        #[rstest]
        fn test_serialize_uses_slash_separator(_tickers: ()) {
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
        #[rstest]
        fn test_serialize_converts_to_uppercase(_tickers: ()) {
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

        /// Checks that non-alphabetic symbols in `base` (like "eth2") are preserved during serialization.
        #[rstest]
        fn test_serialize_preserves_alphanumeric_symbols(_tickers: ()) {
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
}
