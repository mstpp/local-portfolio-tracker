use predicates::prelude::*;

pub const LIST_CMD_COMMON: &str = "\nFound csv files:";
pub const NO_PORTF: &str = "no portfolios";

/// Predicate for the expected "no portfolios" stdout on `list`.
pub fn stdout_no_portfolios() -> impl Predicate<str> {
    predicate::str::contains(LIST_CMD_COMMON).and(predicate::str::contains(NO_PORTF))
}

pub fn stdout_list_has(name: &str) -> impl Predicate<str> {
    predicate::str::contains(LIST_CMD_COMMON).and(predicate::str::contains(name))
}
