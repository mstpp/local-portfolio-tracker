#![allow(dead_code)]
use crate::currency::Ticker;
use crate::trade::{Side, Trade};
use crate::trading_pair::TradingPair;
use anyhow::Result;
use rust_decimal::{Decimal, dec};

#[derive(Debug)]
struct PairPosition {
    pair: TradingPair,
    holdings: Decimal,
    average_price: Decimal,
}

impl PairPosition {
    pub fn new(pair: TradingPair) -> Self {
        PairPosition {
            pair,
            holdings: dec!(0),
            average_price: dec!(0),
        }
    }

    pub fn from_trades(&mut self, trades: Vec<Trade>) -> Result<()> {
        if trades.len() == 0 {
            println!("No trades to calc avg pair position");
            return Ok(());
        }
        let mut holding: Decimal = dec!(0);
        let mut avg: Decimal = dec!(0);
        for tx in trades.into_iter() {
            if tx.pair != self.pair {
                return Err(anyhow::Error::msg(format!(
                    "trading pair missmatch: {:?} {:?}",
                    tx.pair, self.pair
                )));
            }
            match tx.side {
                Side::Buy => {
                    // TODO: fees should be calcualted in the average price
                    avg = (holding * avg + tx.amount * tx.price) / (holding + tx.amount);
                    holding += tx.amount;
                }
                Side::Sell => {
                    if tx.amount > holding {
                        return Err(anyhow::Error::msg(format!(
                            "sell amount ({}) bigger than holdings: {}",
                            &tx.amount, &holding
                        )));
                    } else {
                        holding -= tx.amount;
                    }
                    // TODO realized pnl?
                }
            }
        }

        // update position values
        self.holdings = holding;
        self.average_price = avg;

        Ok(())
    }
}

// Position for a specific ticker,
// calculated in QuoteCurrency defined in settings
#[derive(Debug)]
pub struct Position {
    pub ticker: Ticker,
    pub size: Decimal,
}

#[derive(Debug)]
pub struct Portfolio {
    positions: Vec<Position>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::QuoteCurrency;
    use crate::test_utils::fixtures::{tickers, transactions};
    use rstest::*;

    mod pair_position_tests {
        use std::str::FromStr;

        use super::*;

        #[rstest]
        fn test_new(_tickers: ()) {
            let pair = TradingPair {
                base: Ticker::from_str("BTC").unwrap(),
                quote: QuoteCurrency::Usd,
            };
            let position = PairPosition::new(pair.clone()); // TODO no clone
            assert_eq!(position.average_price, 0.into());
            assert_eq!(position.holdings, 0.into());
            assert_eq!(position.pair, pair);
        }
    }

    mod pair_position_from_trades {
        use super::*;
        use crate::test_utils::helpers::transactions_from;
        use std::str::FromStr;

        fn btc_usd_pair_pos() -> PairPosition {
            let pair = TradingPair {
                base: Ticker::from_str("BTC").unwrap(),
                quote: QuoteCurrency::Usd,
            };
            PairPosition::new(pair)
        }

        #[rstest]
        fn test_empty_vec(_tickers: ()) {
            let mut pair_pos = btc_usd_pair_pos();
            let res = pair_pos.from_trades(vec![]);
            assert!(res.is_ok(), "expected ok, got: {:?}", res);
            assert_eq!(pair_pos.average_price, 0.into());
        }

        #[rstest]
        fn test_2_buy_tx(_tickers: (), transactions: Vec<Trade>) {
            // transactions fixture has 2 tx:
            // buy 1 BTC for 40 000 USD
            // buy 3 BTC for 20 000 USD
            let mut pair_pos = btc_usd_pair_pos();
            let res = pair_pos.from_trades(transactions);
            assert!(res.is_ok());
            assert_eq!(pair_pos.holdings, dec!(4));
            assert_eq!(pair_pos.average_price, dec!(25_000));
        }

        #[rstest]
        fn test_single_tx(_tickers: ()) {
            let csv_data = r#"created_at,pair,side,amount,price,fee
1704883200,BTC/USD,BUY,1.0,40000.00,7.50"#;
            let tx: Vec<Trade> = transactions_from(csv_data);
            let mut pair_pos = btc_usd_pair_pos();
            let res = pair_pos.from_trades(tx);
            assert!(res.is_ok());
            assert_eq!(pair_pos.holdings, dec!(1));
            assert_eq!(pair_pos.average_price, dec!(40_000));
        }

        #[test]
        fn test_invalid_tx_sell_more_than_position() {
            assert!(true);
        }

        #[test]
        fn test_invalid_pair() {
            assert!(true);
        }
    }
}
