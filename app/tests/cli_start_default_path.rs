use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn start_uses_current_dir_when_path_missing() {
    let data = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    // ensure dir exists and is non-empty
    fs::write(cwd.path().join("README.md"), "init\n").unwrap();

    // Run without --path; should use current_dir and succeed
    Command::cargo_bin("belljar")
        .unwrap()
        .current_dir(cwd.path())
        .args(["start", "s1"]) // no --path
        .env("BELLJAR_DATA_DIR", data.path())
        .assert()
        .success();
}
