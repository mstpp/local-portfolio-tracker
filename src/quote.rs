use crate::currency::{CRYPTO, Currency};
use anyhow::{Context, Ok, Result, anyhow};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

static QUOTE_CACHE: LazyLock<Mutex<Option<QuoteCache>>> = LazyLock::new(|| Mutex::new(None));
const CACHE_DURATION: Duration = Duration::from_secs(60);
const GECKO_TICKER_IDS: &str = "data/coingecko.csv";
const GECKO_QUOTE_USD: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd";

struct QuoteCache {
    quotes: HashMap<String, f64>,
    last_updated: Instant,
}

fn get_cached_quotes() -> Result<HashMap<String, f64>> {
    let mut cache = QUOTE_CACHE.lock().unwrap();

    // Check if cache is valid
    let needs_refresh = cache
        .as_ref()
        .map(|c| c.last_updated.elapsed() >= CACHE_DURATION)
        .unwrap_or(true);

    if needs_refresh {
        // Fetch fresh quotes
        let quotes = get_quotes(&*CRYPTO)?;
        *cache = Some(QuoteCache {
            quotes: quotes.clone(),
            last_updated: Instant::now(),
        });
        Ok(quotes)
    } else {
        // Return cached quotes
        Ok(cache.as_ref().unwrap().quotes.clone())
    }
}

pub fn quote_usd(currency: &Currency) -> Result<Decimal> {
    let quotes = get_cached_quotes()?;
    let quote = quotes
        .get(&currency.ticker)
        .ok_or(anyhow!("quote missing"))?;
    Ok(Decimal::from_f64_retain(*quote).ok_or(anyhow!("can't decimal from f64"))?)
}

// needed for deserialization of api return price, which is in format
// {"bitcoin":{"usd":109509},"ethereum":{"usd":3885.46}
// ** currently only suporting USD quotes
#[derive(Debug, serde::Deserialize)]
struct Price {
    usd: f64,
}

/// Obtaining current ticker quotes in USD for ticker list
///
/// Coingecko API accepts ids, while we are using short tickers elsewhere
/// that is why translation from ticker to id is required
/// e.g ticker: BTC -> id: bitcoin
pub fn get_quotes<I, S>(ticks: I) -> Result<HashMap<String, f64>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    // Convert input to Vec<String> for processing
    let tickers: Vec<String> = ticks.into_iter().map(|s| s.as_ref().to_string()).collect();

    // get coingecko coin ids from tickers from coingecko csv table
    let ids = to_ids(&tickers)?;

    // create HashMap id:ticker, since we need to convert them back after getting res from endpoint
    // assumption: to_ids() returns ordered list of ids, based on input ticker list
    let id_ticker_hm: HashMap<String, String> = ids.clone().into_iter().zip(tickers).collect();

    // API endpoint URL with comma separated ids
    let url = GECKO_QUOTE_USD.replace("{}", &ids.join(","));
    let res = reqwest::blocking::get(url)?.json::<HashMap<String, Price>>()?;

    // need to convert back ids to tickers
    let quotes_hm = res
        .into_iter()
        .map(|(id, price)| (id_ticker_hm.get(&id).unwrap().clone(), price.usd))
        .collect();

    Ok(quotes_hm)
}

/// Getting quotes from coingecko api
/// data/coingecko.csv table is holding (id, symbol, name) required for the coingecko API
/// *name is not actually required
#[derive(Debug, Deserialize)]
struct CsvRow {
    id: String,     // long id, e.g. bitcoin, ethereum, etc.
    symbol: String, // short ticker
    #[allow(dead_code)]
    name: String,
}

fn to_ids(tickers: &[String]) -> Result<Vec<String>> {
    let mut reader = csv::Reader::from_path(GECKO_TICKER_IDS)
        .with_context(|| format!("opening {}", GECKO_TICKER_IDS))?;

    let symbol_to_id: HashMap<String, String> = reader
        .deserialize()
        .collect::<Result<Vec<CsvRow>, _>>() // csv::Error -> anyhow::Error via ?
        .context("parsing coins CSV")?
        .into_iter()
        .map(|coin| (coin.symbol.to_ascii_uppercase(), coin.id))
        .collect();

    let ids = tickers
        .iter()
        .map(|t| {
            let key = t.to_ascii_uppercase();
            symbol_to_id
                .get(&key)
                .cloned()
                .ok_or_else(|| anyhow!("Ticker not found: {}", t)) // <- produce an Error, not String
        })
        .collect::<Result<Vec<_>>>()?; // inferred as Result<Vec<String>, anyhow::Error>

    Ok(ids)
}

//
// = = = = = TEST = = = = =
//

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vs {
        ($($s:expr),* $(,)?) => {
            vec![$($s.to_string()),*]
        };
    }

    #[test]
    fn test_to_ids_single_match() {
        let tickers = vs!["BTC"];
        let ids = to_ids(&tickers).unwrap();
        assert_eq!(ids, vec!["bitcoin"]);
    }

    #[test]
    fn test_to_ids_multiple_matches() {
        // do not use SOL, SOL is the same ticker for Solana and wrapped solana
        // similar for DOGE
        let tickers = vs!["eth", "ADA"];
        let ids = to_ids(&tickers).unwrap();
        assert_eq!(ids, vec!["ethereum", "cardano"]);
    }

    #[test]
    fn test_to_ids_no_match() {
        let tickers = vs!["aaabtccc"];
        // v1
        // let res = to_ids(&tickers);
        // assert!(res.is_err());
        // let err_msg = res.unwrap_err().to_string();
        // assert!(err_msg.contains("Ticker not found: aaabtccc"));
        // v2
        let err = to_ids(&tickers).unwrap_err();
        assert!(err.to_string().contains("Ticker not found: aaabtccc"))
    }

    #[test]
    fn test_ticker_to_id_order() {
        let tickers = vs!["ADA", "eth", "BTC"];
        assert_eq!(
            to_ids(&tickers).unwrap(),
            vec!["cardano", "ethereum", "bitcoin"]
        );
        let tickers = vs!["TRX", "eth", "ada", "BTC"];
        assert_eq!(
            to_ids(&tickers).unwrap(),
            vec!["tron", "ethereum", "cardano", "bitcoin"]
        );
    }
}
