use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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

fn make_tmux_shim(has_session_exit: i32) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("tmux.log");
    let shim = dir.path().join("tmux");
    let script = format!(
        "#!/usr/bin/env bash\necho \"$@\" >> {}\nif [ \"$1\" = has-session ]; then exit {}; fi\nexit 0\n",
        log.display(), has_session_exit
    );
    fs::write(&shim, script).unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    (dir, log)
}

fn prepend_path(dir: &Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn open_and_send_use_tmux() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();
    let (shim_dir, log) = make_tmux_shim(0); // has-session succeeds

    // start session
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["start", "s1", "--path"])
        .arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success();

    // open attaches
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["open", "s1"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success();

    // send a command
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["send", "s1", "echo", "hi"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success();

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("attach-session"));
    assert!(logged.contains("send-keys"));
}

#[test]
fn control_center_creates_windows() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();
    let (shim_dir, log) = make_tmux_shim(1); // has-session fails => new-session

    // two sessions
    for s in ["s1", "s2"] {
        Command::cargo_bin("belljar")
            .unwrap()
            .args(["start", s, "--path"])
            .arg(repo.path())
            .env("BELLJAR_DATA_DIR", data.path())
            .assert()
            .success();
    }

    Command::cargo_bin("belljar")
        .unwrap()
        .arg("control-center")
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success();

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("new-session"));
    assert!(logged.contains("new-window"));
    assert!(logged.contains("select-layout"));
    assert!(logged.contains("attach-session"));
}

#[test]
fn workspace_placeholder_and_rm_all() {
    let data = TempDir::new().unwrap();
    let repo = init_git_repo();

    // create two sessions
    for s in ["a", "b"] {
        Command::cargo_bin("belljar")
            .unwrap()
            .args(["start", s, "--path"])
            .arg(repo.path())
            .env("BELLJAR_DATA_DIR", data.path())
            .assert()
            .success();
    }

    // workspace placeholder
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["workspace", "ls"])
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success();

    // rm all
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["rm", "all"])
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed a").and(predicate::str::contains("removed b")));
}
