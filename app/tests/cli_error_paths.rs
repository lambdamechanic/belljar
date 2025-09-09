use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn start_invalid_path_fails() {
    Command::cargo_bin("belljar").unwrap()
        .args(["start", "s1", "--path", "/definitely/not/here"]) 
        .assert()
        .failure()
        .stderr(predicate::str::contains("path does not exist"));
}

#[test]
fn open_without_tmux_prints_fallback() {
    let data = TempDir::new().unwrap();
    let repo = TempDir::new().unwrap();
    // create session directly via CLI
    Command::cargo_bin("belljar").unwrap()
        .args(["start", "s1", "--path"]).arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert().success();

    // clear PATH so tmux is missing
    Command::cargo_bin("belljar").unwrap()
        .args(["open", "s1"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", "/nonexistent")
        .assert()
        .success()
        .stdout(predicate::str::contains("tmux not found; cd "));
}

#[test]
fn send_all_with_tmux_missing_reports_errors_but_succeeds() {
    let data = TempDir::new().unwrap();
    let repo = TempDir::new().unwrap();
    // create two sessions
    for s in ["a", "b"] {
        Command::cargo_bin("belljar").unwrap()
            .args(["start", s, "--path"]).arg(repo.path())
            .env("BELLJAR_DATA_DIR", data.path())
            .assert().success();
    }
    Command::cargo_bin("belljar").unwrap()
        .args(["send", "all", "echo", "hi"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", "/nonexistent")
        .assert()
        .success();
}

#[test]
fn control_center_no_sessions() {
    let data = TempDir::new().unwrap();
    Command::cargo_bin("belljar").unwrap()
        .arg("control-center")
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no sessions to show"));
}
