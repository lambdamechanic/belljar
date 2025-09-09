use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn start_fails_when_data_dir_unwritable() {
    // On typical systems, writing under / is not allowed for normal users.
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1"])
        .env("BELLJAR_DATA_DIR", "/")
        .assert()
        .failure()
        .stderr(predicate::str::contains("create session failed"));
}
