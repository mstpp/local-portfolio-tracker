use predicates::prelude::*;

#[allow(dead_code)]
pub fn stdout_list_has(name: &str) -> impl Predicate<str> {
    predicate::str::contains("CSV file name").and(predicate::str::contains(name))
}
