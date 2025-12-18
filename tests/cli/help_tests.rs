// cargo t --test help_cmd_tests
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn help_flag_displays_usage_information_full() {
    let expected_help = "\
CSV Portfolio Tracker

A command-line tool to manage CSV-based investment portfolios, calculate PnL, and generate performance reports.

Usage: portfolio-tracker [OPTIONS] <COMMAND>

Commands:
  list    List all portfolios [aliases: l, ls]
  new     Create new portfolio
  show    Show all transactions from portfolio
  report  Report portfolio PnL
  add-tx  Add transaction to portfolio
  help    Print this message or the help of the given subcommand(s)

Options:
  -p, --portfolio-dir <PORTFOLIO_DIR>
          

  -h, --help
          Print help (see a summary with '-h')
";

    for flag in ["--help", "help"] {
        let mut cmd = cargo_bin_cmd!("portfolio-tracker");
        cmd.arg(flag)
            .assert()
            .success()
            .stdout(predicate::str::diff(expected_help)) // Exact match with diff
            .stderr(predicate::str::is_empty());
    }
}

#[test]
fn help_flag_displays_usage_information_short() {
    let expected_help = "\
CSV Portfolio Tracker

Usage: portfolio-tracker [OPTIONS] <COMMAND>

Commands:
  list    List all portfolios [aliases: l, ls]
  new     Create new portfolio
  show    Show all transactions from portfolio
  report  Report portfolio PnL
  add-tx  Add transaction to portfolio
  help    Print this message or the help of the given subcommand(s)

Options:
  -p, --portfolio-dir <PORTFOLIO_DIR>  
  -h, --help                           Print help (see more with '--help')
";

    let mut cmd = cargo_bin_cmd!("portfolio-tracker");
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::diff(expected_help)) // Exact match with diff
        .stderr(predicate::str::is_empty());
}

#[test]
fn show_help_for_new_cmd() {
    let expected = "\
Create new portfolio

Usage: portfolio-tracker new [OPTIONS] --name <NAME>

Options:
  -n, --name <NAME>          
      --currency <CURRENCY>  
  -h, --help                 Print help
";
    let mut cmd = cargo_bin_cmd!("portfolio-tracker");
    cmd.args(["new", "-h"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::diff(expected));
}
