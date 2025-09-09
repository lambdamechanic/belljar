#![cfg(not(coverage))]
#![allow(unexpected_cfgs)]
#![cfg(not(coverage))]
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo() -> tempfile::TempDir {
    let td = TempDir::new().unwrap();
    let repo = td.path();
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("init")
        .status()
        .unwrap()
        .success());
    // set identity for commit on CI
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "user.email", "ci@example.com"])
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "user.name", "CI"])
        .status()
        .unwrap()
        .success());
    std::fs::write(repo.join("README.md"), "init\n").unwrap();
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["add", "."])
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["commit", "-m", "init"])
        .status()
        .unwrap()
        .success());
    td
}

#[test]
fn cli_ls_start_rm_cycle() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();

    // start
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1", "--path"])
        .arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created session: s1"));

    // ls contains s1
    Command::cargo_bin("belljar")
        .unwrap()
        .arg("ls")
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("s1\t"));

    // rm s1
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["rm", "s1"])
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed s1"));

    // ls should not include s1 anymore
    Command::cargo_bin("belljar")
        .unwrap()
        .arg("ls")
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("s1").not());
}

#[test]
fn cli_checkout_creates_session() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();

    // create a branch to checkout
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["checkout", "-b", "fx"])
        .status()
        .unwrap()
        .success());

    // checkout
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["checkout", "fx", "--path"])
        .arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("checked out: fx -> fx"));
}
