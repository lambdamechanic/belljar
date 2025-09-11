use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn make_tmux_shim() -> TempDir {
    let dir = TempDir::new().unwrap();
    let shim = dir.path().join("tmux");
    // Log arguments and always succeed
    let script = "#!/usr/bin/env bash\necho \"$@\" >> tmux.log\nexit 0\n";
    fs::write(&shim, script).unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    dir
}

fn prepend_path(dir: &Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn workspace_start_open_and_rm() {
    let data = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    // create two repo dirs under root
    fs::create_dir_all(root.path().join("frontend")).unwrap();
    fs::create_dir_all(root.path().join("backend")).unwrap();

    let shim_dir = make_tmux_shim();

    // start workspace and open
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["workspace", "start", "dev-ws", "--path"])
        .arg(root.path())
        .args(["--repos", "frontend,backend", "--open"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success()
        .stdout(predicate::str::contains("created workspace: dev-ws"));

    // ls shows the workspace
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["workspace", "ls"])
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("dev-ws\t"));

    // open again (tmux shim ensures calls succeed)
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["workspace", "open", "dev-ws"])
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(shim_dir.path()))
        .assert()
        .success();

    // rm workspace
    Command::cargo_bin("belljar")
        .unwrap()
        .args(["workspace", "rm", "dev-ws"])
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed workspace dev-ws"));
}
