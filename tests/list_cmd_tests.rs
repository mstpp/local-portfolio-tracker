mod common;
mod fixtures;

use crate::fixtures::TestContext;
use predicates::prelude::*;

#[test]
fn list_on_empty_workspace_shows_no_portfolios() {
    let ctx = TestContext::new();

    let expected_stdout = "\
+---------------+------------+
| CSV file name | Created at |
+---------------+------------+
";

    ctx.cmd()
        .arg("list")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::diff(expected_stdout));
}
