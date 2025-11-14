use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn help_flag_displays_usage_information() {
    // Test both long and short flags
    for flag in ["--help", "-h"] {
        // let mut cmd = Command::cargo_bin("portfolio-tracker").expect("binary should build");
        let mut cmd = cargo_bin_cmd!("portfolio-tracker");

        cmd.arg(flag)
            .assert()
            .success() // exit code 0
            .stdout(predicate::str::contains("Usage"))
            .stdout(predicate::str::contains("Commands"))
            .stdout(predicate::str::contains("Options"))
            .stderr(predicate::str::is_empty());
    }
}
