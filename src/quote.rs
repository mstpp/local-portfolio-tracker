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

// v1
// moving to use struct with serde,
// readability and complexity for using tuple on too many places
// type Coin = (String, String, String); // (id, symbol, name)

// v1:
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Err = Box<dyn std::error::Error>; // TODO use anyhow::Result instead?

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
pub fn get_quotes(tickers: Vec<String>) -> Result<HashMap<String, f64>, Err> {
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

// Learning 📖
// v1 [bug]
//
// fn to_ids(tickers: Vec<String>) -> Result<Vec<String>, Err> {
//     let mut reader = csv::Reader::from_path("data/coingecko.csv")?;
//     let data: Vec<Coin> = reader
//         .deserialize()
//         .collect::<Result<Vec<Coin>, csv::Error>>()?;
//     let mut ids: Vec<String> = Vec::new();
//     // ⛔️ inefficient: going over whole csv data instead of over tickers
//     for (id, symbol, _) in data.iter() {
//         // ⚠️⛔️🐛 ORDER is not preserved, find based on order in the csv file
//         if tickers.contains(&symbol) {
//             ids.push(id.to_string());
//         }
//     }
//     // ⛔️ TODO missing validation if all ids found for all tickers
//     Ok(ids)
// }

#[derive(Debug, Deserialize)]
struct Coin {
    id: String,
    symbol: String,
    name: String, // TODO not used (dead code)
}

// Learning 📖
// v1 [bug]
//
// .ok_or_else(|| format!("Ticker not found: {}", t))
// cannot accept String type, but needs a Error type!
//
// fn to_ids_v1(tickers: &[String]) -> Result<Vec<String>, Err> {
//     let mut reader = csv::Reader::from_path(COINGECKO_TAB)?;

//     let symbol_to_id: HashMap<String, String> = reader
//         .deserialize()
//         .collect::<csv::Result<Vec<Coin>>>()?
//         .into_iter()
//         .map(|coin| (coin.symbol.to_uppercase(), coin.id))
//         .collect();

//     let ids: Vec<String> = tickers
//         .iter()
//         .map(|t| {
//             symbol_to_id
//                 .get(&t.to_uppercase())
//                 .cloned()
//                 .ok_or_else(|| format!("Ticker not found: {}", t)) // ⚠️⛔️🐛
//         })
//         .collect::<Result<Vec<_>, _>>()?;
//     Ok(ids)
// }

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

    // Learning 📖
    // v1
    // purpose: avoid adding .to_sting() in each vec memeber
    //
    // fn vs(v: &[&str]) -> Vec<String> {
    //     v.iter().map(|i| i.to_string()).collect()
    // }

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
        let tickers = vs!["eth", "SOL"];
        let ids = to_ids(&tickers).unwrap();
        assert_eq!(ids, vec!["ethereum", "solana"]);
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
        let tickers = vs!["SOL", "eth", "BTC"];
        assert_eq!(
            to_ids(&tickers).unwrap(),
            vec!["solana", "ethereum", "bitcoin"]
        );
        let tickers = vs!["SOL", "eth", "ada", "BTC"];
        assert_eq!(
            to_ids(&tickers).unwrap(),
            vec!["solana", "ethereum", "cardano", "bitcoin"]
        );
    }
}
