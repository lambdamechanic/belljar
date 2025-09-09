use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn rm_unknown_session() {
    let data = TempDir::new().unwrap();
    Command::cargo_bin("belljar").unwrap()
        .args(["rm", "nope"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no such session: nope"));
}

#[test]
fn open_unknown_session() {
    let data = TempDir::new().unwrap();
    Command::cargo_bin("belljar").unwrap()
        .args(["open", "nope"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no such session: nope"));
}

#[test]
fn send_unknown_session() {
    let data = TempDir::new().unwrap();
    Command::cargo_bin("belljar").unwrap()
        .args(["send", "nope", "echo"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no such session: nope"));
}
