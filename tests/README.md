# E2E test scenarios 

1.	Bootstrap / Help Output
Verify --help and -h show all commands, flags, argument types, and examples without panics.
2.	List Portfolios (none yet)
Run list on a fresh workspace; prints an empty state message and exits 0.
3.	Create New Portfolio (happy path)
new <name> creates directory/files; rerun list shows the new portfolio.



	4.	Create Portfolio That Already Exists
Second new <name> fails gracefully with a clear “already exists” error and non-zero exit.
	5.	Show Trades on Empty Portfolio
show <name> on a newly created portfolio renders headers and “no trades” message.
	6.	Add BUY Transaction (simple)
add-tx --name <name> --ticker TICK --side buy --qty 10 --price 5 --fee 0.5 appends a row; show displays it correctly.
	7.	Add SELL Transaction (simple)
Add a sell; show reflects both rows in correct chronological order and formatting.
	8.	Add Multiple Transactions / Aggregation Order
Add several buys/sells out of chronological order; ensure persisted order or normalized sort is as specified by the app.
	9.	Report Holdings (single ticker)
report <name> computes quantities, average cost, unrealized PnL, and fees correctly for one ticker.
	10.	Report Holdings (multiple tickers)
Mix of tickers; verify per-ticker aggregation and a portfolio total line (if supported).
	11.	Decimal Parsing & Precision
Large and fractional qty/price/fee (e.g., 0.000123, 1234567.89); values are stored and reported without rounding errors.
	12.	Validation: Zero/Negative Qty or Price
Reject qty <= 0 or price <= 0 with a clear user-facing error (no file writes).
	13.	Validation: Side Argument
Reject invalid --side values (case/enum); accept documented variants (e.g., buy/sell).
	14.	Validation: Ticker / Trading Pair Format
Accept valid tickers or pairs (e.g., AAPL, BTC-USD); reject malformed ones with a helpful message.
	15.	Fees Handling
Ensure fee is optional or required as designed; totals and PnL incorporate fees exactly once.
	16.	Show Trades: CSV Integrity
Manually corrupt a row (missing column, bad decimal); show and report surface a readable error pointing to the line.
	17.	Cross-Platform Line Endings
Files with \n vs \r\n read/write cleanly; output remains consistent.
	18.	Idempotent Runs
Running a read-only command twice (list, show, report) yields identical results and no file changes.
	19.	Concurrency / File Lock Graceful Error
Simulate file lock or in-use CSV; write operations fail with a clear “locked/busy” message, read ops behave as designed.
	20.	Unknown Portfolio Name
show/report/add-tx on non-existent --name errors cleanly with exit code != 0.
	21.	Case Sensitivity & Canonicalization
Creating MyBook vs mybook: behavior matches spec (either treated as distinct or prevented) consistently across commands.
	22.	Default Data Location & Custom Path (if supported)
Verify default storage path; if a flag/env to override exists, all commands honor it.
	23.	Quotes Source Happy Path (if report pulls quotes)
With network or stubbed quotes, report shows current price and PnL; verify ticker-to-quote mapping.
	24.	Quotes Failure / Offline Mode
When quotes fetch fails (network/API), report degrades gracefully (cached/last price/explicit error), with non-zero exit only if specified.
	25.	Large Portfolio Performance
Import or generate 10k+ rows; show and report complete within reasonable time and memory, formatting intact.
	26.	UTF-8 Tickers / Descriptions
Support non-ASCII tickers or notes if allowed; output encoding remains valid.
	27.	CLI Exit Codes
Success paths return 0; validation, missing files, and IO/parse errors return non-zero, consistent across commands.
	28.	Version / Build Info (if exposed)
--version prints semantic version and exits 0.
	29.	Backup / Atomic Writes
After add-tx, verify either atomic write (temp + rename) or backup file exists as designed; no partial writes on crash simulation.
	30.	Timezone / Timestamp Handling (if present)
If trades carry timestamps, ensure parsing, ordering, and display in the intended timezone/format.