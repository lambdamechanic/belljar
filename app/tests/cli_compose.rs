use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo_with_compose() -> (tempfile::TempDir, PathBuf) {
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
    fs::write(repo.join("README.md"), "init\n").unwrap();
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
    // add compose file
    let bj = repo.join(".belljar/compose");
    fs::create_dir_all(&bj).unwrap();
    let cf = bj.join("svc.yml");
    fs::write(&cf, "services: {}\n").unwrap();
    (td, cf)
}

fn prepend_path(dir: &Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

fn make_docker_shim_logging() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("docker.log");
    let shim = dir.path().join("docker");
    let script = format!(
        "#!/usr/bin/env bash\necho \"$@\" >> {}\nexit 0\n",
        log.display()
    );
    fs::write(&shim, script).unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    (dir, log)
}

#[test]
fn start_with_compose_runs_up() {
    let data = TempDir::new().unwrap();
    let (repo_td, cf) = init_git_repo_with_compose();
    let (shim_dir, log) = make_docker_shim_logging();

    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1", "--path"])
        .arg(repo_td.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stdout(predicate::str::contains("[compose up]"));

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("compose"));
    assert!(logged.contains("up -d"));
    assert!(logged.contains(cf.file_name().unwrap().to_string_lossy().as_ref()));
}

#[test]
fn checkout_with_compose_runs_up() {
    let data = TempDir::new().unwrap();
    let (repo_td, cf) = init_git_repo_with_compose();
    let (shim_dir, log) = make_docker_shim_logging();
    // create branch to checkout
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo_td.path())
        .args(["checkout", "-b", "fx"])
        .status()
        .unwrap()
        .success());

    Command::cargo_bin("belljar")
        .unwrap()
        .args(["checkout", "fx", "--path"])
        .arg(repo_td.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stdout(predicate::str::contains("[compose up]"));

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("up -d"));
    assert!(logged.contains(cf.file_name().unwrap().to_string_lossy().as_ref()));
}

#[test]
fn rm_session_triggers_down() {
    let data = TempDir::new().unwrap();
    let (repo_td, _cf) = init_git_repo_with_compose();
    let (shim_dir, log) = make_docker_shim_logging();

    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1", "--path"])
        .arg(repo_td.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success();

    Command::cargo_bin("belljar")
        .unwrap()
        .args(["rm", "s1"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stdout(predicate::str::contains("removed s1"));

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("down -v"));
}

#[test]
fn rm_all_triggers_down_for_all() {
    let data = TempDir::new().unwrap();
    let (repo_td, _cf) = init_git_repo_with_compose();
    let (shim_dir, log) = make_docker_shim_logging();

    for s in ["a", "b"] {
        Command::cargo_bin("belljar")
            .unwrap()
            .args(["start", s, "--path"])
            .arg(repo_td.path())
            .env("BELLJAR_DATA_DIR", data.path())
            .env("PATH", prepend_path(shim_dir.path()))
            .assert()
            .success();
    }

    Command::cargo_bin("belljar")
        .unwrap()
        .args(["rm", "all"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stdout(predicate::str::contains("removed a").and(predicate::str::contains("removed b")));

    let logged = fs::read_to_string(&log).unwrap();
    let count = logged.matches(" down -v").count();
    assert!(
        count >= 2,
        "expected at least two down invocations, got {count} in: {logged}"
    );
}
