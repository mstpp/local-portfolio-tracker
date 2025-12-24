use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

/// Supported fiat currencies
pub static FIAT: LazyLock<HashSet<&str>> = LazyLock::new(|| HashSet::from(["USD", "EUR", "CAD"]));
/// Supported stable coins
pub static STABLES: LazyLock<HashSet<&str>> =
    LazyLock::new(|| HashSet::from(["USDC", "USDT", "USDS", "DAI", "USDE"]));
/// Supported cryptocurrencies
// It's possible to add automatically generated list from Coingecko API, but for now,
// it's enough to just manually define non-exhaustive list
pub static CRYPTO: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    HashSet::from([
        "BTC", "ETH", "XRP", "BNB", "SOL", "TRX", "DOGE", "ADA", "BCH", "LINK", "HYPE", "LEO",
        "WETH", "XLM", "XMR", "SUI", "AVAX", "LTC", "HBAR", "ZEC", "SHIB", "CRO", "TON", "DOT",
        "UNI", "MNT", "AAVE", "TAO", "BGB", "M", "S", "OKB", "NEAR", "ASTER", "ETC", "ICP", "PI",
        "PEPE", "RAIN", "PUMP", "ONDO", "HTX", "JLP", "KAS",
    ])
});

/// A validated currency ticker with automatic type classification.
///
/// Tickers are automatically normalized (trimmed and uppercased).
///
/// # Examples
/// ```
/// use portfolio_tracker::currency::{Currency, CurrencyType};
///
/// let btc = Currency::new("BTC").unwrap();
/// assert_eq!(btc.ticker(), "BTC");
/// assert_eq!(btc.currency_type(), CurrencyType::Crypto);
///
/// // Normalization works automatically
/// let eth = Currency::new(" eth ").unwrap();
/// assert_eq!(eth.ticker(), "ETH");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Currency {
    ticker: String,
}

impl Currency {
    pub fn new(ticker: &str) -> Result<Self> {
        let ticker = normalize_ticker(ticker);

        classify_ticker(&ticker).ok_or_else(|| {
            anyhow!(
                "Unsupported ticker '{}'. Valid examples: BTC, ETH, USD, USDC",
                ticker
            )
        })?;

        Ok(Self { ticker })
    }

    pub fn ticker(&self) -> &str {
        &self.ticker
    }

    pub fn currency_type(&self) -> CurrencyType {
        classify_ticker(&self.ticker).expect("Currency ticker was validated at construction")
    }

    pub fn is_valid(ticker: &str) -> bool {
        let normalized = normalize_ticker(ticker);
        classify_ticker(&normalized).is_some()
    }

    /// Returns an iterator over all supported cryptocurrency tickers.
    pub fn supported_crypto() -> impl Iterator<Item = &'static str> {
        CRYPTO.iter().copied()
    }

    /// Returns an iterator over all supported fiat currency tickers.
    pub fn supported_fiat() -> impl Iterator<Item = &'static str> {
        FIAT.iter().copied()
    }

    /// Returns an iterator over all supported stablecoin tickers.
    pub fn supported_stables() -> impl Iterator<Item = &'static str> {
        STABLES.iter().copied()
    }
}

impl Default for Currency {
    fn default() -> Self {
        Currency::new("USD").expect("USD must be valid currency")
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ticker)
    }
}

impl FromStr for Currency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

// Custom Serialize to match custom Deserialize
impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.ticker)
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ticker = String::deserialize(deserializer)?;
        Currency::new(&ticker).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyType {
    Crypto,
    Fiat,
    StableCoin,
}

/// Classifies a ticker into its currency type
fn classify_ticker(ticker: &str) -> Option<CurrencyType> {
    if FIAT.contains(ticker) {
        Some(CurrencyType::Fiat)
    } else if STABLES.contains(ticker) {
        Some(CurrencyType::StableCoin)
    } else if CRYPTO.contains(ticker) {
        Some(CurrencyType::Crypto)
    } else {
        None
    }
}

fn normalize_ticker(s: &str) -> String {
    s.trim().to_ascii_uppercase()
}

// ---------------------------------------------------------
// ---------------------------------------------------------
// TESTS
// ---------------------------------------------------------
// ---------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_str_eq};
    use rstest::rstest;

    // === Currency Creation Tests ===

    #[rstest]
    #[case("BTC", "BTC")]
    #[case("btc", "BTC")]
    #[case(" btc ", "BTC")]
    #[case("\tETH\n", "ETH")]
    #[case("  UsD  ", "USD")]
    fn test_currency_creation_with_normalization(#[case] input: &str, #[case] expected: &str) {
        let currency = Currency::new(input).unwrap();
        assert_eq!(currency.ticker(), expected);
    }

    #[rstest]
    #[case("INVALID")]
    #[case("abtc")]
    #[case("NOTACOIN")]
    #[case("")]
    #[case("   ")]
    #[case("\t")]
    fn test_currency_creation_invalid_tickers(#[case] invalid_ticker: &str) {
        let result = Currency::new(invalid_ticker);
        assert!(
            result.is_err(),
            "Expected error for ticker: '{}'",
            invalid_ticker
        );
    }

    #[test]
    fn test_error_message_format() {
        let result = Currency::new("INVALID");
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("INVALID"));
        assert!(err_msg.contains("Unsupported ticker"));
    }

    // === Currency Type Classification ===

    #[rstest]
    #[case("BTC", CurrencyType::Crypto)]
    #[case("ETH", CurrencyType::Crypto)]
    #[case("XRP", CurrencyType::Crypto)]
    #[case("USD", CurrencyType::Fiat)]
    #[case("EUR", CurrencyType::Fiat)]
    #[case("CAD", CurrencyType::Fiat)]
    #[case("USDC", CurrencyType::StableCoin)]
    #[case("USDT", CurrencyType::StableCoin)]
    #[case("DAI", CurrencyType::StableCoin)]
    fn test_currency_type_classification(#[case] ticker: &str, #[case] expected: CurrencyType) {
        let currency = Currency::new(ticker).unwrap();
        assert_eq!(currency.currency_type(), expected);
    }

    // === Validation Tests ===

    #[rstest]
    #[case("BTC")]
    #[case("btc")]
    #[case(" BTC ")]
    #[case("USD")]
    #[case("USDC")]
    #[case("\tETH\n")]
    fn test_is_valid_true(#[case] ticker: &str) {
        assert!(Currency::is_valid(ticker));
    }

    #[rstest]
    #[case("INVALID")]
    #[case("")]
    #[case("   ")]
    #[case("NOTREAL")]
    fn test_is_valid_false(#[case] ticker: &str) {
        assert!(!Currency::is_valid(ticker));
    }

    // === Normalization Tests ===

    #[rstest]
    #[case("ABC", "ABC")]
    #[case(" AbC ", "ABC")]
    #[case("\n abc\t ", "ABC")]
    #[case("lowercase", "LOWERCASE")]
    #[case("  MiXeD CaSe  ", "MIXED CASE")]
    fn test_normalize_ticker(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(normalize_ticker(input), expected);
    }

    // === FromStr Implementation ===

    #[test]
    fn test_from_str_success() {
        let btc: Currency = "BTC".parse().unwrap();
        assert_eq!(btc.ticker(), "BTC");

        let eth: Currency = " eth ".parse().unwrap();
        assert_eq!(eth.ticker(), "ETH");
    }

    #[test]
    fn test_from_str_failure() {
        let result: Result<Currency> = "INVALID".parse();
        assert!(result.is_err());
    }

    // === Display Implementation ===

    #[rstest]
    #[case("BTC", "BTC")]
    #[case("USD", "USD")]
    #[case("USDC", "USDC")]
    fn test_display(#[case] ticker: &str, #[case] expected: &str) {
        let currency = Currency::new(ticker).unwrap();
        assert_eq!(format!("{}", currency), expected);
    }

    // === Default Implementation ===

    #[test]
    fn test_default() {
        let default = Currency::default();
        assert_eq!(default.ticker(), "USD");
        assert_eq!(default.currency_type(), CurrencyType::Fiat);
    }

    // === Clone, PartialEq, Eq, Hash Tests ===

    #[test]
    fn test_clone() {
        let btc = Currency::new("BTC").unwrap();
        let btc_clone = btc.clone();
        assert_eq!(btc, btc_clone);
    }

    #[test]
    fn test_equality() {
        let btc1 = Currency::new("BTC").unwrap();
        let btc2 = Currency::new("btc").unwrap(); // Different case
        let eth = Currency::new("ETH").unwrap();

        assert_eq!(btc1, btc2); // Same after normalization
        assert_ne!(btc1, eth); // Different currencies
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let btc = Currency::new("BTC").unwrap();
        map.insert(btc.clone(), "Bitcoin");

        // Same ticker should retrieve the value
        let btc2 = Currency::new("btc").unwrap();
        assert_eq!(map.get(&btc2), Some(&"Bitcoin"));
    }

    // === Serde Tests ===

    #[test]
    fn test_serde_serialize() {
        let btc = Currency::new("BTC").unwrap();
        let json = serde_json::to_string(&btc).unwrap();
        assert_eq!(json, r#""BTC""#); // Note: string, not object
    }

    #[test]
    fn test_serde_deserialize_valid() {
        let json = r#""BTC""#; // String, not object
        let currency: Currency = serde_json::from_str(json).unwrap();
        assert_eq!(currency.ticker(), "BTC");
        assert_eq!(currency.currency_type(), CurrencyType::Crypto);
    }

    #[test]
    fn test_serde_deserialize_with_normalization() {
        let json = r#""btc""#; // String, not object
        let currency: Currency = serde_json::from_str(json).unwrap();
        assert_eq!(currency.ticker(), "BTC");
    }

    #[test]
    fn test_serde_deserialize_invalid() {
        let json = r#""INVALID""#; // String, not object
        let result: Result<Currency, _> = serde_json::from_str(json);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unsupported ticker"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = Currency::new("ETH").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
    // === Edge Cases ===

    #[test]
    fn test_empty_ticker_error() {
        let result = Currency::new("");
        assert!(result.is_err());
        assert_str_eq!(
            result.unwrap_err().to_string(),
            "Unsupported ticker ''. Valid examples: BTC, ETH, USD, USDC"
        );
    }

    #[test]
    fn test_whitespace_only_ticker_error() {
        let result = Currency::new("   ");
        assert!(result.is_err());
        assert_str_eq!(
            result.unwrap_err().to_string(),
            "Unsupported ticker ''. Valid examples: BTC, ETH, USD, USDC"
        );
    }

    #[test]
    fn test_unicode_ticker() {
        // Should fail - we only support ASCII
        let result = Currency::new("₿TC");
        assert!(result.is_err());
    }

    // === CurrencyType Tests ===

    #[test]
    fn test_currency_type_copy() {
        let ct1 = CurrencyType::Crypto;
        let ct2 = ct1; // Copy, not move
        assert_eq!(ct1, ct2); // ct1 still valid
    }

    #[test]
    fn test_currency_type_equality() {
        assert_eq!(CurrencyType::Crypto, CurrencyType::Crypto);
        assert_ne!(CurrencyType::Crypto, CurrencyType::Fiat);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_normalization_idempotent(s in "[a-zA-Z]{1,10}") {
            let norm1 = normalize_ticker(&s);
            let norm2 = normalize_ticker(&norm1);
            prop_assert_eq!(norm1, norm2);
        }

        #[test]
        fn test_valid_currency_roundtrip(
            ticker in prop::sample::select(vec!["BTC", "ETH", "USD", "USDC", "USDT"])
        ) {
            let currency = Currency::new(ticker).unwrap();
            let json = serde_json::to_string(&currency)?;
            let deserialized: Currency = serde_json::from_str(&json)?;
            prop_assert_eq!(currency, deserialized);
        }
    }
}
