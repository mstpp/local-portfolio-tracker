//#[path = "common.rs"] //this is when no dir structure,
// not playing well with analyzer, it sees dead code
// mod common;
use crate::common::stdout_list_has;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

pub struct TestContext {
    temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        // println!("DEBUG: TEMPDIR: {:?}", &temp_dir);
        Self { temp_dir }
    }

    pub fn cmd(&self) -> assert_cmd::Command {
        let mut cmd = cargo_bin_cmd!("portfolio-tracker");
        cmd.env("CSVPT_DATA_DIR", self.temp_dir.path());
        cmd
    }

    pub fn create_portfolio(&self, name: &str) {
        self.cmd()
            .args(["new", "--name", name])
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
    }

    pub fn portfolio_path(&self, name: &str) -> std::path::PathBuf {
        self.temp_dir.path().join(name).with_extension("csv")
    }

    pub fn assert_portfolio_exists(&self, name: &str) {
        let path = self.portfolio_path(name);
        assert!(
            std::fs::metadata(&path)
                .expect("portfolio should exist")
                .is_file(),
            "Portfolio '{}' should exist at {:?}",
            name,
            path
        );
    }

    pub fn assert_list_contains(&self, name: &str) {
        self.cmd()
            .arg("list")
            .assert()
            .success()
            .stdout(stdout_list_has(name))
            .stderr(predicate::str::is_empty());
    }

    pub fn show_empty_portfolio(&self, name: &str) {
        self.cmd()
            .args(["show", "--name", name])
            .assert()
            .success()
            .code(0)
            .stdout(predicate::str::contains("No trades found"))
            .stderr(predicate::str::is_empty());
    }

    pub fn add_tx_buy_btc(&self, portfolio: &str, qty: &str, price: &str, fee: &str) {
        self.cmd()
            .args([
                "add-tx", "--name", portfolio, "--ticker", "BTC/USD", "--side", "BUY", "--qty",
                qty, "--price", price, "--fee", fee,
            ])
            .assert()
            .success()
            .code(0)
            .stdout(predicate::str::contains(
                "Added transaction to portfolio csv file:",
            ))
            .stderr(predicate::str::is_empty());
    }

    pub fn report(&self, portfolio: &str) {
        self.cmd()
            .args(["report", "--name", portfolio])
            .assert()
            .success()
            .code(0)
            .stderr(predicate::str::is_empty())
            .stdout(predicate::str::contains("Total PnL USD:"));
    }
}
