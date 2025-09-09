use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

fn prepend_path(dir: &PathBuf) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn worktree_setup_failure_is_warned() {
    let data = TempDir::new().unwrap();
    let repo = TempDir::new().unwrap();

    // git shim: rev-parse succeeds -> considered a git repo; worktree add fails
    let shim_dir = TempDir::new().unwrap();
    let git = shim_dir.path().join("git");
    let script = "#!/usr/bin/env bash\nif [ \"$3\" = rev-parse ]; then exit 0; fi\nif [ \"$3\" = worktree ]; then exit 1; fi\nexit 0\n";
    fs::write(&git, script).unwrap();
    let mut perm = fs::metadata(&git).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&git, perm).unwrap();

    Command::cargo_bin("belljar").unwrap()
        .args(["start", "s1", "--path"]).arg(repo.path())
        .env("BELLJAR_DATA_DIR", data.path())
        .env("PATH", prepend_path(&shim_dir.path().to_path_buf()))
        .assert()
        .success()
        .stderr(predicate::str::contains("worktree setup failed"));
}

