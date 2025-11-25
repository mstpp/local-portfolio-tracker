#![allow(dead_code)]
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CoinInfo {
    #[serde(rename = "id")]
    name: String,
    #[serde(rename = "symbol", deserialize_with = "uppercase_string")]
    ticker: String,
    #[serde(rename = "name")]
    long_name: String,
}

fn uppercase_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.to_uppercase())
}

/// Get tickers for first 250 coins by mcap from Coingecko API
/// Requirements:
/// - CGECKO_API_KEY in environment vars
pub fn tickers() -> Result<()> {
    let api_key =
        std::env::var("CGECKO_API_KEY").with_context(|| "CGECKO_API_KEY env var missing")?;
    let url = "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=250&page=1";

    let client = Client::new();
    let response = client
        .get(url)
        .header("x-cg-demo-api-key", api_key)
        .send()?;

    let coins: Vec<CoinInfo> = response.json()?;

    let file_path = "./data/coingecko.csv";
    let file = std::fs::File::create(file_path)?; // override if exists
    let mut wrt = csv::Writer::from_writer(file);

    for coin in coins.iter() {
        wrt.serialize(coin)?;
    }

    wrt.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    tickers()?;
    Ok(())
}
