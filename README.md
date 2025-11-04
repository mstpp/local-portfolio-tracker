# local-portfolio-tracker
Local CLI that tracks portfolio in CSV file


#### CLI usage examples

```bash
# long commands/args
cargo r -- help
cargo r -- list
cargo r -- show example
cargo r -- new novi-portfolio
cargo r -- report example
cargo r -- add-tx --name example --ticker BTC/USD --side BUY --qty 0.2 --price 99320 --fee 12

# short commands/args
cargo r -- l
cargo r -- ls
cargo r -- s example
cargo r -- n novi-portfolio
cargo r -- r example
cargo r -- a -n example -t BTC/USD --side BUY -q 0.1 -p 99000 -f 12
```
