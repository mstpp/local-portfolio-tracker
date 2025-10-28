#![allow(dead_code)]
// getting quotes from coingecko api
use serde::Deserialize;
use std::collections::HashMap;

// type Coin = (String, String, String); // (id, symbol, name)
type Err = Box<dyn std::error::Error>;
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, serde::Deserialize)]
struct Price {
    usd: f64,
}

pub fn get_q(tickers: Vec<String>) -> Result<HashMap<String, f64>, Err> {
    // get coingecko coin ids from short tickers
    let ids = to_ids(&tickers)?;
    // let ids = to_ids(tickers.clone())?;
    // println!("DEBUG: ids vec {:?}", &ids);

    // create HashMap id:ticker
    let id_ticker_hm: HashMap<String, String> = ids.clone().into_iter().zip(tickers).collect();
    // println!("DEBUG: id ticker hashmap {:?}", &id_ticker_hm);

    // comma separated string from vec
    let ids_comma = ids.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        ids_comma
    );
    // println!("DEBUG: {}", &url);
    // api returns e.g.
    // {"bitcoin":{"usd":109509},"ethereum":{"usd":3885.46}
    // that is why Price struct for deser is used
    let res = reqwest::blocking::get(url)?.json::<HashMap<String, Price>>()?;
    // println!("DEBUG: response: {:?}", &res);
    let quotes_hm = res
        .into_iter()
        .map(|(id, price)| (id_ticker_hm.get(&id).unwrap().clone(), price.usd))
        .collect();
    Ok(quotes_hm)
}

// fn to_ids(tickers: Vec<String>) -> Result<Vec<String>, Err> {
//     // data/coingecko.csv holding id, symbol, name
//     // api required id, and symbol is the short uppercase ticker e.g. BTC, ETH
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
    name: String,
}

// fn to_ids(tickers: Vec<String>) -> Result<Vec<String>> {
//     // Load coingecko CSV
//     let mut reader = csv::Reader::from_path("data/coingecko.csv")?;
//     let data: Vec<Coin> = reader.deserialize().collect::<Result<_>>()?;

//     // Map symbol (uppercase) -> id
//     let mut symbol_to_id = HashMap::new();
//     for coin in data {
//         symbol_to_id.insert(coin.symbol.to_uppercase(), coin.id);
//     }

//     // Preserve order of input `tickers`
//     let mut ids = Vec::with_capacity(tickers.len());
//     for t in tickers.clone() {
//         if let Some(id) = symbol_to_id.get(&t.to_uppercase()) {
//             ids.push(id.clone());
//         }
//     }

//     // What if we don't find a ticker?
//     assert!(tickers.len() == ids.len(), "Some id missing");

//     Ok(ids)
// }

// fn to_ids(tickers: Vec<String>) -> Result<Vec<String>, Err> {
fn to_ids(tickers: &[String]) -> Result<Vec<String>, Err> {
    let mut reader = csv::Reader::from_path("data/coingecko.csv")?;

    let symbol_to_id: HashMap<String, String> = reader
        .deserialize()
        .collect::<csv::Result<Vec<Coin>>>()?
        .into_iter()
        .map(|coin| (coin.symbol.to_uppercase(), coin.id))
        .collect();

    let ids: Vec<String> = tickers
        .iter()
        .map(|t| {
            symbol_to_id
                .get(&t.to_uppercase())
                .cloned()
                .ok_or_else(|| format!("Ticker not found: {}", t))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // let tickers = vec!["BTC".to_string()];
        // let tickers = vs(&["BTC"]);
        let tickers = vs!["BTC"];
        let ids = to_ids(&tickers).unwrap();
        assert_eq!(ids, vec!["bitcoin"]);
    }

    #[test]
    fn test_to_ids_multiple_matches() {
        // let tickers = vec!["eth".to_string(), "SOL".to_string()];
        let tickers = vs!["eth", "SOL"];
        let ids = to_ids(&tickers).unwrap();
        assert_eq!(ids, vec!["ethereum", "solana"]);
    }

    #[test]
    fn test_to_ids_no_match() {
        let tickers = vs!["aaabtccc"];
        let res = to_ids(&tickers);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("Ticker not found: aaabtccc"));
    }

    #[test]
    fn test_order() {
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
