# E2E test scenarios 

## Cli tests 

```bash
cargo t --test cli
```

### Help 

```bash
cargo t --test cli help_tests
```

- [x] Help `--help` and `help` (long), validate complete exact stdout
- [x] Help `-h` (short) command, validate complete exact stdout
- [x] Long and short help for command: new
- [] Long and short help for command: list
- [] Long and short help for command: show
- [] Long and short help for command: report
- [] Long and short help for command: add-tx

### List Portfolios

```bash
cargo t --test cli list_tests
```

- [x] List Portfolios (none yet) - prints an empty state, header only message and exits 0, exact stdout match
- [] List when file is not a CSV file - should be ignored 
- [] List after created empty CSV file - should be displayed w/o csv extension

### Create New Portfolio


```bash
cargo t --test cli new_tests
```

- [x] Create New Portfolio (happy path)
	- new <name> creates directory/files,
	- rerun list shows the new portfolio,
	- content: it has correct first line comment
	- content: it has correct header
- [x] Create Portfolio That Already Exists - Second new returns "File exists" msg
- [x] Create Portfolio with non-default currency (EUR)

### Show Trades

```bash
cargo t --test cli show_tests
```

- [x] Show Trades on Empty Portfolio - renders headers and "no trades" message
- [x] Show portfolio for EUR (non default USD) currency (one tx)
- [] Base currency comment is in EUR, multiple trades present
- [] Decimal Parsing & Precision - Large and fractional qty/price/fee (e.g., 0.000123, 1234567.89); values are stored and reported without rounding errors
- [] Validation: Zero/Negative Qty or Price - Reject qty <= 0 or price <= 0 with a clear user-facing error (no file writes).

### Add Tx

```bash
cargo t --test cli add_tx_tests
```

- [x] Add BUY Transaction (simple)
add-tx --name <name> --ticker TICK --side buy --qty 10 --price 5 --fee 0.5 appends a row; show displays it correctly.
- [] Add SELL Transaction (simple)
Add a sell; show reflects both rows in correct chronological order and formatting.
- [] Add Multiple Transactions / Aggregation Order
Add several buys/sells out of chronological order; ensure persisted order or normalized sort is as specified by the app.

### Report

```bash
cargo t --test report_cmd_tests
```

- [x] Report Holdings (single ticker)
report <name> computes quantities, average cost, unrealized PnL, and fees correctly for one ticker.
- [] Report Holdings (multiple tickers)
Mix of tickers; verify per-ticker aggregation and a portfolio total line (if supported).


### Other 

- [] Validation: Ticker / Trading Pair Format - Accept valid tickers or pairs (e.g., AAPL, BTC-USD); reject malformed ones with a helpful message.
- [] Fees Handling - Ensure fee is optional or required as designed; totals and PnL incorporate fees exactly once.
- [] Show Trades: CSV Integrity Manually corrupt a row (missing column, bad decimal); show and report surface a readable error pointing to the line.
- [] Cross-Platform Line Endings Files with \n vs \r\n read/write cleanly; output remains consistent.
- [] Idempotent Runs Running a read-only command twice (list, show, report) yields identical results and no file changes.
- [] Concurrency / File Lock Graceful Error Simulate file lock or in-use CSV; write operations fail with a clear “locked/busy” message, read ops behave as designed.
- [] Unknown Portfolio Name show/report/add-tx on non-existent --name errors cleanly with exit code != 0.
- [] Case Sensitivity & Canonicalization Creating MyBook vs mybook: behavior matches spec (either treated as distinct or prevented) consistently across commands.
- [] Default Data Location & Custom Path (if supported) Verify default storage path; if a flag/env to override exists, all commands honor it.
- [] Quotes Source Happy Path (if report pulls quotes) With network or stubbed quotes, report shows current price and PnL; verify ticker-to-quote mapping.
- [] Quotes Failure / Offline Mode When quotes fetch fails (network/API), report degrades gracefully (cached/last price/explicit error), with non-zero exit only if specified.
- [] Large Portfolio Performance Import or generate 10k+ rows; show and report complete within reasonable time and memory, formatting intact.
- [] UTF-8 Tickers / Descriptions Support non-ASCII tickers or notes if allowed; output encoding remains valid.
- [] CLI Exit Codes Success paths return 0; validation, missing files, and IO/parse errors return non-zero, consistent across commands.
- [] Backup / Atomic Writes After add-tx, verify either atomic write (temp + rename) or backup file exists as designed; no partial writes on crash simulation.
- [] Timezone / Timestamp Handling (if present) If trades carry timestamps, ensure parsing, ordering, and display in the intended timezone/format.