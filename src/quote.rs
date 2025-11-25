#![allow(dead_code)]
/// Getting quotes from coingecko api
/// data/coingecko.csv table is holding (id, symbol, name) required for the coingecko API
/// *name is not actually required
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::collections::HashMap;

const COINGECKO_TAB: &str = "data/coingecko.csv";
const CG_QUOTE_USD_API: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd";

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
pub fn get_quotes(tickers: Vec<String>) -> Result<HashMap<String, f64>> {
    // get coingecko coin ids from tickers from coingecko csv table
    let ids = to_ids(&tickers)?;

    // create HashMap id:ticker, since we need to convert them back after getting res from endpoint
    // assumption: to_ids() returns ordered list of ids, based on input ticker list
    let id_ticker_hm: HashMap<String, String> = ids.clone().into_iter().zip(tickers).collect();

    // API endpoint URL with comma separated ids
    let url = CG_QUOTE_USD_API.replace("{}", &ids.join(","));
    let res = reqwest::blocking::get(url)?.json::<HashMap<String, Price>>()?;

    // need to convert back ids to tickers
    let quotes_hm = res
        .into_iter()
        .map(|(id, price)| (id_ticker_hm.get(&id).unwrap().clone(), price.usd))
        .collect();

    Ok(quotes_hm)
}

#[derive(Debug, Deserialize)]
struct Coin {
    id: String,
    symbol: String,
    name: String, // TODO not used (dead code)
}

fn to_ids(tickers: &[String]) -> anyhow::Result<Vec<String>> {
    let mut reader = csv::Reader::from_path(COINGECKO_TAB)
        .with_context(|| format!("opening {}", COINGECKO_TAB))?;

    let symbol_to_id: HashMap<String, String> = reader
        .deserialize()
        .collect::<Result<Vec<Coin>, _>>() // csv::Error -> anyhow::Error via ?
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
