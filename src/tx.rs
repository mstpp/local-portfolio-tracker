use crate::currency::Currency;
use anyhow::{Context, Result};
use rust_decimal::Decimal;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tx {
    pub buy: Currency,
    pub buy_size: Decimal,
    pub sell: Currency,
    pub sell_size: Decimal,
}

impl Tx {
    // buy btc example: "0.01 btc for 100.0 usd"
    // sell btc example: "10000 usd for 1 btc"
    pub fn parse(s: &str) -> Result<Self> {
        // reduce sold amount
        let mut amount_iter = s
            .split_ascii_whitespace()
            .filter_map(|s| s.parse::<Decimal>().ok());
        let buy = amount_iter
            .next()
            .ok_or(anyhow::format_err!("missing buy amount"))?;
        let sell = amount_iter
            .next()
            .ok_or(anyhow::format_err!("missing sell amount"))?;

        // incerase for buy amout
        let str_split: Vec<String> = s
            .split_ascii_whitespace()
            .map(|s| s.to_ascii_uppercase())
            .collect();

        let buy_currency =
            Currency::from_ticker(str_split[1].as_str()).with_context(|| "parse buy ticker err")?;
        let sell_currency = Currency::from_ticker(str_split[4].as_str())
            .with_context(|| "prase sell ticker err")?;

        Ok(Tx {
            buy: buy_currency,
            buy_size: buy,
            sell: sell_currency,
            sell_size: sell,
        })
    }
}
