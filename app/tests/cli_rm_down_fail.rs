use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

fn init_repo_with_compose() -> tempfile::TempDir {
    let td = TempDir::new().unwrap();
    let repo = td.path();
    std::fs::create_dir_all(repo.join(".belljar/compose")).unwrap();
    std::fs::write(repo.join(".belljar/compose/svc.yml"), "services: {}\n").unwrap();
    td
}

fn prepend_path(dir: &PathBuf) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn rm_single_down_failure_warns() {
    let data = TempDir::new().unwrap();
    let repo = init_repo_with_compose();
    // docker shim: any call fails
    let shim_dir = TempDir::new().unwrap();
    let docker = shim_dir.path().join("docker");
    fs::write(&docker, "#!/usr/bin/env bash\nexit 1\n").unwrap();
    let mut perm = fs::metadata(&docker).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&docker, perm).unwrap();

    // create session
    Command::cargo_bin("belljar").unwrap()
        .args(["start", "s1", "--path"]).arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .assert().success();

    // rm s1 with failing docker
    Command::cargo_bin("belljar").unwrap()
        .args(["rm", "s1"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(&shim_dir.path().to_path_buf()))
        .assert()
        .success()
        .stderr(predicate::str::contains("warning: compose down failed for s1"));
}

#[test]
fn send_all_ensure_session_failure_warns() {
    let data = TempDir::new().unwrap();
    let repo = TempDir::new().unwrap();
    for s in ["a", "b"] {
        Command::cargo_bin("belljar").unwrap()
            .args(["start", s, "--path"]).arg(repo.path())
            .env("BELLJAR_DATA_DIR", data.path())
            .assert().success();
    }
    // tmux shim: has-session fails and new-session fails to force ensure_session error
    let shim_dir = TempDir::new().unwrap();
    let tmux = shim_dir.path().join("tmux");
    fs::write(&tmux, "#!/usr/bin/env bash\nif [ \"$1\" = has-session ]; then exit 1; fi\nif [ \"$1\" = new-session ]; then exit 1; fi\nexit 0\n").unwrap();
    let mut perm = fs::metadata(&tmux).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&tmux, perm).unwrap();

    Command::cargo_bin("belljar").unwrap()
        .args(["send", "all", "echo"]) 
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(&shim_dir.path().to_path_buf()))
        .assert()
        .success()
        .stderr(predicate::str::contains("ensure session a failed"));
}

