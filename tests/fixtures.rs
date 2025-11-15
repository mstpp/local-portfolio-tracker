use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

pub struct TestContext {
    temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("failed to create temp dir"),
        }
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
}
