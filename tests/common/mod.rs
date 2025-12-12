use predicates::prelude::*;

pub fn stdout_list_has(name: &str) -> impl Predicate<str> {
    predicate::str::contains("CSV file name").and(predicate::str::contains(name))
}
