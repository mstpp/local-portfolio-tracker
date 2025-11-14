use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

mod common;
use common::{stdout_list_has, stdout_no_portfolios};

#[test]
fn list_on_empty_workspace_shows_no_portfolios() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut cmd = cargo_bin_cmd!("portfolio-tracker");

    cmd.arg("list")
        .env("CSVPT_DATA_DIR", temp_dir.path())
        .assert()
        .success()
        .stdout(stdout_no_portfolios())
        .stderr(predicate::str::is_empty());
}

#[test]
fn create_new_portfolio_then_list_shows_it() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let name = "alpha";

    // 1) Create portfolio
    cargo_bin_cmd!("portfolio-tracker")
        .args(["new", "--name", name])
        .env("CSVPT_DATA_DIR", temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    // 2) Filesystem: portfolio csv file should exist
    let created = temp_dir.path().join(name).with_extension("csv");
    assert!(
        std::fs::metadata(&created)
            .expect("portfolio should exist")
            .is_file()
    );

    // 3) list should show the new portfolio
    cargo_bin_cmd!("portfolio-tracker")
        .arg("list")
        .env("CSVPT_DATA_DIR", temp_dir.path())
        .assert()
        .success()
        .stdout(stdout_list_has(name))
        .stderr(predicate::str::is_empty());
}
