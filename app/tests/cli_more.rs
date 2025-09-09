use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn prepend_path(dir: &Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn ls_no_sessions() {
    let data = TempDir::new().unwrap();
    Command::cargo_bin("belljar")
        .unwrap()
        .arg("ls")
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no sessions"));
}

fn make_tmux_shim(has_session_exit: i32, attach_exit: i32, send_exit: i32) -> TempDir {
    let dir = TempDir::new().unwrap();
    let shim = dir.path().join("tmux");
    let script = format!(
        "#!/usr/bin/env bash\nif [ \"$1\" = has-session ]; then exit {has_session_exit}; fi\nif [ \"$1\" = attach-session ]; then exit {attach_exit}; fi\nif [ \"$1\" = send-keys ]; then exit {send_exit}; fi\nexit 0\n"
    );
    fs::write(&shim, script).unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    dir
}

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
    assert!(Command::new("git").arg("-C").arg(repo).args(["config","user.email","ci@example.com"]).status().unwrap().success());
    assert!(Command::new("git").arg("-C").arg(repo).args(["config","user.name","CI"]).status().unwrap().success());
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
    td
}

#[test]
fn send_error_path_reports_but_exits_success() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();
    // create session
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1", "--path"])
        .arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success();

    // tmux shim: has-session ok, attach ok, send-keys fails
    let shim_dir = make_tmux_shim(0, 0, 1);
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["send", "s1", "echo", "hi"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stderr(predicate::str::contains("send failed"));
}

#[test]
fn open_attach_failure_is_reported() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1", "--path"])
        .arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success();

    // tmux shim: has-session ok; attach fails
    let shim_dir = make_tmux_shim(0, 1, 0);
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["open", "s1"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stderr(predicate::str::contains("failed to attach"));
}

#[test]
fn control_center_attach_failure_reported() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();
    for s in ["a", "b"] {
        Command::cargo_bin("belljar")
            .unwrap()
            .args(["start", s, "--path"])
            .arg(repo.path())
            .env("BELLJAR_DATA_DIR", data.path())
            .assert()
            .success();
    }
    // has-session fails so new-session created, but attach fails
    let shim_dir = make_tmux_shim(1, 1, 0);
    Command::cargo_bin("belljar")
        .unwrap()
        .arg("control-center")
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stderr(predicate::str::contains("failed to attach control center"));
}
