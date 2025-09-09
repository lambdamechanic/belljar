use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo() -> tempfile::TempDir {
    let td = TempDir::new().unwrap();
    let repo = td.path();
    assert!(Command::new("git").arg("-C").arg(repo).arg("init").status().unwrap().success());
    std::fs::write(repo.join("README.md"), "init\n").unwrap();
    assert!(Command::new("git").arg("-C").arg(repo).args(["add", "."]).status().unwrap().success());
    assert!(Command::new("git").arg("-C").arg(repo).args(["commit", "-m", "init"]).status().unwrap().success());
    td
}

#[test]
fn version_and_help() {
    Command::cargo_bin("belljar").unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("belljar"));

    Command::cargo_bin("belljar").unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn start_ls_rm_cycle() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();

    Command::cargo_bin("belljar").unwrap()
        .args(["start", "s1", "--path"]).arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success();

    Command::cargo_bin("belljar").unwrap()
        .arg("ls")
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("s1\t"));

    Command::cargo_bin("belljar").unwrap()
        .args(["rm", "s1"])
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed s1"));
}

#[test]
fn checkout_flow() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();
    assert!(Command::new("git").arg("-C").arg(repo.path()).args(["checkout", "-b", "fx"]).status().unwrap().success());

    Command::cargo_bin("belljar").unwrap()
        .args(["checkout", "fx", "--path"]).arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("checked out: fx -> fx"));
}

