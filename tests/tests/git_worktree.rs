use std::process::Command;
use tempfile::TempDir;

#[test]
fn git_worktree_creates_dir() {
    // Skip if git not available
    if Command::new("git").arg("--version").output().is_err() {
        return;
    }

    let repo_td = TempDir::new().unwrap();
    let repo = repo_td.path();
    // init git repo
    assert!(Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("init")
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git").arg("-C").arg(repo).args(["config","user.email","ci@example.com"]).status().unwrap().success());
    assert!(Command::new("git").arg("-C").arg(repo).args(["config","user.name","CI"]).status().unwrap().success());
    // create initial commit
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

    // ensure worktree
    let wt =
        par_core::git::ensure_worktree(repo, "wt-test", &Some("wt-test".into())).expect("worktree");
    assert!(wt.exists(), "worktree path should exist");
}
