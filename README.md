# local-portfolio-tracker
Local CLI that tracks portfolio in CSV file


#### CLI usage examples

```bash
# long commands/args
cargo r --bin portfolio-tracker -- help
cargo r --bin portfolio-tracker -- list
cargo r --bin portfolio-tracker -- show --name example
cargo r --bin portfolio-tracker -- new --name new-pfl
cargo r --bin portfolio-tracker -- report --name example
cargo r --bin portfolio-tracker -- add-tx --name new-pfl --ticker BTC/USD --side BUY --qty 0.2 --price 99320 --fee 12

# short commands/args
cargo r --bin portfolio-tracker -- l
cargo r --bin portfolio-tracker -- ls
cargo r --bin portfolio-tracker -- s -n example
cargo r --bin portfolio-tracker -- n -n new-pfl
cargo r --bin portfolio-tracker -- r -n example
cargo r --bin portfolio-tracker -- add-tx -n new-pfl -t BTC/USD --side BUY -q 0.1 -p 99000 -f 12
```

### Generate csv file with tickers 

It will produce `data/coingecko.csv` with first 250 tickers by mcap: 

`cargo r --bin get_coingecko_tickers`
