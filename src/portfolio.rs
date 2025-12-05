use crate::currency::{Currency, CurrencyType, QuoteCurrency};
use crate::quote::tmp::quote_usd;
use crate::trade::Trade;
use crate::tx::Tx;
use anyhow::Result;
use rust_decimal::{Decimal, dec};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct Portfolio {
    pub positions: HashMap<Currency, Position>,
    // pub transactions: Vec<Tx>,
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            positions: HashMap::new(),
            // transactions: Vec::new(),
        }
    }

    pub fn deposit(&mut self, currency: Currency, amount: Decimal) -> Result<()> {
        let pos = self
            .positions
            .entry(currency.clone())
            .or_insert(Position::new(currency.clone()));

        pos.balance += amount;

        // For USD, cost_base should equal balance (1:1)
        if currency == Currency::from_ticker("USD").unwrap() {
            pos.cost_base += amount;
        } else {
            pos.cost_base += amount * quote_usd(&currency); // get real quote TODO
        }

        Ok(())
    }

    // withdraw currency TODO

    // buy side - sell side
    // 1   BTC for 100_000 USD
    pub fn add_tx(&mut self, tx: Tx) -> Result<()> {
        // Reduce sell position
        let sell_pos = self
            .positions
            .entry(tx.sell.clone())
            .or_insert(Position::new(tx.sell.clone()));

        if sell_pos.balance < tx.sell_size {
            anyhow::bail!("Insufficient balance");
        }

        // Calculate proportional cost basis being sold
        let avg_cost = if tx.sell.ticker == "USD".to_string() {
            dec!(1)
        } else {
            // (sell_pos.cost_base / sell_pos.balance).round_dp(2)
            sell_pos.cost_base / sell_pos.balance
        };
        let cost_basis_sold = avg_cost * tx.sell_size;

        sell_pos.balance -= tx.sell_size;
        sell_pos.cost_base -= cost_basis_sold;

        // Add buy position
        let buy_pos = self
            .positions
            .entry(tx.buy.clone())
            .or_insert(Position::new(tx.buy.clone()));

        buy_pos.balance += tx.buy_size;
        buy_pos.cost_base += cost_basis_sold;

        // self.transactions.push(tx);

        Ok(())
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut pf = Portfolio::new();
        let mut reader = csv::Reader::from_path(path)?;

        for result in reader.deserialize::<Trade>() {
            let trade = result?;
            // TODO temp: if buy in USD tx, auto deposit
            if trade.pair.quote == QuoteCurrency::Usd {
                let amount = trade.amount * trade.price + trade.fee;
                pf.deposit(Currency::from_ticker("USD")?, amount)?;
            }
            pf.add_tx(trade.to_tx()?)?;
        }

        Ok(pf)
    }

    pub fn print_unrealized_pnl<P: AsRef<Path>>(path: P) -> Result<()> {
        let pf = Portfolio::from_csv(path)?;

        for (currency, position) in pf.positions.iter() {
            if currency.currency_type == CurrencyType::Crypto {
                println!("{}", currency.ticker);
                println!(
                    "{} PnL: {:.2} %",
                    position.balance,
                    position.balance * quote_usd(currency)
                        - position.cost_base / position.cost_base
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub currency: Currency,
    pub balance: Decimal,
    pub cost_base: Decimal, // USD
}

impl Position {
    pub fn new(currency: Currency) -> Self {
        Position {
            currency,
            balance: dec!(0),
            cost_base: dec!(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::sync::LazyLock;

    static BTC: LazyLock<Currency> =
        LazyLock::new(|| Currency::from_ticker("BTC").expect("BTC should be valid"));

    static USD: LazyLock<Currency> =
        LazyLock::new(|| Currency::from_ticker("USD").expect("USD should be valid"));

    // Test fixtures for common scenarios
    #[fixture]
    fn portfolio_with_1m_usd() -> Portfolio {
        let mut pf = Portfolio::new();
        // Initial deposit: $1M USD
        pf.deposit(USD.clone(), dec!(1000_000)).unwrap();
        pf
    }

    #[fixture]
    fn portfolio_with_10_btc() -> Portfolio {
        let mut pf = Portfolio::new();
        // Initial deposit: 10 BTC
        pf.deposit(BTC.clone(), dec!(10)).unwrap();
        pf
    }

    // ========== Deposit Tests ==========

    #[rstest]
    fn test_deposit_creates_position_with_correct_balance(portfolio_with_1m_usd: Portfolio) {
        let pos = portfolio_with_1m_usd.positions.get(&USD).unwrap();
        assert_eq!(pos.balance, dec!(1_000_000));
    }

    #[rstest]
    fn test_deposit_sets_initial_cost_basis(portfolio_with_10_btc: Portfolio) {
        let pos = portfolio_with_10_btc.positions.get(&BTC).unwrap();
        let btc_val = dec!(10) * quote_usd(&BTC);
        assert_eq!(pos.cost_base, btc_val);
    }

    // ========== Buy Tests ==========

    #[test]
    fn test_buying_btc_decreases_usd_and_increases_btc_balance() {
        let mut pf = portfolio_with_1m_usd();

        // Buy 10 BTC for $100K (should have 10 BTC and $900K remaining)
        let res = pf.add_tx(Tx::parse("10 btc for 100000 usd").unwrap());

        assert!(res.is_ok());
        assert_eq!(pf.positions.get(&BTC).unwrap().balance, dec!(10));
        assert_eq!(pf.positions.get(&USD).unwrap().balance, dec!(900_000));
    }

    #[test]
    fn test_buying_btc_increases_cost_basis() {
        let mut pf = portfolio_with_1m_usd();

        // Buy 1 BTC for $150K (total cost basis should be $150K for 1 BTC)
        pf.add_tx(Tx::parse("1 btc for 150000 usd").unwrap())
            .unwrap();

        let btc_pos = pf.positions.get(&BTC).unwrap();
        assert_eq!(btc_pos.balance, dec!(1));
        assert_eq!(btc_pos.cost_base, dec!(150_000));
        let usd_pos = pf.positions.get(&USD).unwrap();
        assert_eq!(usd_pos.balance, dec!(850_000));
    }

    #[test]
    fn test_buying_with_insufficient_funds_returns_error() {
        let mut pf = Portfolio::new();

        // Deposit only $1,000
        pf.deposit(USD.clone(), dec!(1000)).unwrap();

        // Try to buy 10 BTC for $100K (should fail)
        let res = pf.add_tx(Tx::parse("10 btc for 100000 usd").unwrap());

        assert!(res.is_err());
        // Balance should remain unchanged
        assert_eq!(pf.positions.get(&USD).unwrap().balance, dec!(1000));
    }

    #[test]
    fn test_buying_without_any_funds_returns_error() {
        let mut pf = Portfolio::new();

        // Try to buy without any USD position
        let res = pf.add_tx(Tx::parse("1 btc for 10000 usd").unwrap());

        assert!(res.is_err());
        // Should not create BTC position
        assert!(pf.positions.get(&BTC).is_none());
    }

    // ========== Sell Tests ==========

    #[test]
    fn test_selling_btc_increases_usd_and_decreases_btc_balance() {
        let mut pf = portfolio_with_10_btc();

        // Sell 1 BTC for $20K (should have 9 BTC and $20K USD)
        let res = pf.add_tx(Tx::parse("20000 usd for 1 btc").unwrap());

        assert!(res.is_ok());
        assert_eq!(pf.positions.get(&BTC).unwrap().balance, dec!(9));
        assert_eq!(pf.positions.get(&USD).unwrap().balance, dec!(20000));
    }

    #[test]
    fn test_selling_btc_decreases_cost_basis_proportionally() {
        let mut pf = portfolio_with_1m_usd();

        // Deposit 10 BTC with cost basis of $100K ($10K per BTC)
        pf.add_tx(Tx::parse("10 btc for 100000 usd").unwrap())
            .unwrap();

        // Sell 5 BTC (should reduce cost basis by half to $50K)
        pf.add_tx(Tx::parse("60000 usd for 5 btc").unwrap())
            .unwrap();

        let btc_pos = pf.positions.get(&BTC).unwrap();
        assert_eq!(btc_pos.balance, dec!(5));
        assert_eq!(btc_pos.cost_base, dec!(50000));
    }

    #[test]
    fn test_selling_more_than_owned_returns_error() {
        let mut pf = portfolio_with_10_btc();

        // Try to sell 10 BTC (should fail)
        let res = pf.add_tx(Tx::parse("100000 usd for 11 btc").unwrap());

        assert!(res.is_err());
        // Balance should remain unchanged
        assert_eq!(pf.positions.get(&BTC).unwrap().balance, dec!(10));
    }

    #[test]
    fn test_selling_entire_position_removes_asset() {
        let mut pf = portfolio_with_10_btc();

        // Sell all 10 BTC
        pf.add_tx(Tx::parse("60000 usd for 10 btc").unwrap())
            .unwrap();

        let btc_pos = pf.positions.get(&BTC).unwrap();
        assert_eq!(btc_pos.balance, dec!(0));
        assert_eq!(btc_pos.cost_base, dec!(0));
    }

    // ========== Parameterized Tests ==========

    #[rstest]
    #[case(
        "10 btc for 100000 usd",     // Buy transaction
        dec!(10),                    // Expected BTC balance
        dec!(900_000)                // Expected USD balance
    )]
    #[case(
        "5 btc for 50000 usd",
        dec!(5),
        dec!(950_000)
    )]
    #[case(
        "1 btc for 10000 usd",
        dec!(1),
        dec!(990_000)
    )]
    fn test_buy_transactions_update_balances_correctly(
        #[case] buy_tx: &str,
        #[case] expected_btc: Decimal,
        #[case] expected_usd: Decimal,
    ) {
        let mut pf = portfolio_with_1m_usd();

        pf.add_tx(Tx::parse(buy_tx).unwrap()).unwrap();

        assert_eq!(pf.positions.get(&BTC).unwrap().balance, expected_btc);
        assert_eq!(pf.positions.get(&USD).unwrap().balance, expected_usd);
    }

    #[rstest]
    #[case(
        "20000 usd for 1 btc",      // Sell transaction
        dec!(9),                     // Expected BTC balance
        dec!(20000)                  // Expected USD balance
    )]
    #[case(
        "30000 usd for 2 btc",
        dec!(8),
        dec!(30000)
    )]
    #[case(
        "50000 usd for 5.15 btc",
        dec!(4.85),
        dec!(50000)
    )]
    fn test_sell_transactions_update_balances_correctly(
        #[case] sell_tx: &str,
        #[case] expected_btc: Decimal,
        #[case] expected_usd: Decimal,
    ) {
        let mut pf = portfolio_with_10_btc();

        pf.add_tx(Tx::parse(sell_tx).unwrap()).unwrap();

        assert_eq!(pf.positions.get(&BTC).unwrap().balance, expected_btc);
        assert_eq!(pf.positions.get(&USD).unwrap().balance, expected_usd);
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_multiple_buys_accumulate_cost_basis() {
        let mut pf = portfolio_with_1m_usd();

        // Buy 1 BTC for $40K
        pf.add_tx(Tx::parse("1 btc for 40000 usd").unwrap())
            .unwrap();
        // Buy another 1 BTC for $50K (total cost should be $90K for 2 BTC)
        pf.add_tx(Tx::parse("1 btc for 50000 usd").unwrap())
            .unwrap();

        let btc_pos = pf.positions.get(&BTC).unwrap();
        assert_eq!(btc_pos.balance, dec!(2));
        assert_eq!(btc_pos.cost_base, dec!(90000));
        assert_eq!(pf.positions.get(&USD).unwrap().balance, dec!(910_000));
    }

    #[test]
    fn test_buy_then_sell_then_buy_again() {
        let mut pf = portfolio_with_1m_usd();

        // Buy 2 BTC for $100K (cost basis: $100K for 2 BTC) USD: $900K
        pf.add_tx(Tx::parse("2 btc for 100000 usd").unwrap())
            .unwrap();

        assert_eq!(pf.positions.get(&BTC).unwrap().cost_base, dec!(100_000));

        // Sell 1 BTC for $60K (cost basis should be $50K for 1 BTC) USD: $960K
        pf.add_tx(Tx::parse("60000 usd for 1 btc").unwrap())
            .unwrap();
        assert_eq!(pf.positions.get(&BTC).unwrap().cost_base, dec!(50000));

        // Buy 1 BTC for $55K (cost basis should be $105K for 2 BTC) USD: $905K
        pf.add_tx(Tx::parse("1 btc for 55000 usd").unwrap())
            .unwrap();
        assert_eq!(pf.positions.get(&BTC).unwrap().balance, dec!(2));
        assert_eq!(pf.positions.get(&USD).unwrap().balance, dec!(905_000));
        assert_eq!(pf.positions.get(&BTC).unwrap().cost_base, dec!(105_000));
    }

    #[test]
    fn test_exact_balance_transactions() {
        let mut pf = portfolio_with_1m_usd();

        // Buy using exactly all USD (should succeed and leave 0 USD)
        let res = pf.add_tx(Tx::parse("10 btc for 1000000 usd").unwrap());

        assert!(res.is_ok());
        assert_eq!(pf.positions.get(&USD).unwrap().balance, dec!(0));
        assert_eq!(pf.positions.get(&BTC).unwrap().balance, dec!(10));
    }
}
