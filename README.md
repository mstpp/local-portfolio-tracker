# local-portfolio-tracker
Local CLI that tracks portfolio in CSV file


#### CLI usage examples

```bash
cargo r -- help
cargo r -- list
cargo r -- show example
cargo r -- new novi-portfolio
cargo r -- report example
cargo r -- add-tx trade example BTC/USD BUY 1 115000 50

cargo r -- l
cargo r -- ls
cargo r -- s example
cargo r -- n novi-portfolio
cargo r -- r example
cargo r -- a trade example BTC/USD BUY 1 115000 50
```

```bash
local-portfolio-tracker help (h)
local-portfolio-tracker list-portfolios (ls)
local-portfolio-tracker show (s) <portfolio-name>
local-portfolio-tracker new (n) <portfolio-name> 
local-portfolio-tracker report (r) <portfolio-name> 
local-portfolio-tracker add-tx (a) <portfolio-name> <tx-data>
```
