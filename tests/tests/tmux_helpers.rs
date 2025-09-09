use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

fn prepend_path(dir: &PathBuf) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn tmux_helpers_call_binary() {
    // Create tmux shim
    let shim_dir = TempDir::new().unwrap();
    let log = shim_dir.path().join("tmux.log");
    let shim = shim_dir.path().join("tmux");
    let script = format!("#!/usr/bin/env bash\necho \"$@\" >> {}\nif [ \"$1\" = has-session ]; then exit 1; fi\nexit 0\n", log.display());
    fs::write(&shim, script).unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    std::env::set_var("PATH", prepend_path(&shim_dir.path().to_path_buf()));

    // Minimal session
    let s = par_core::Session {
        id: "id".into(),
        label: "lab".into(),
        repo_path: std::env::current_dir().unwrap(),
        branch: None,
        worktree_path: None,
        compose_project: "proj".into(),
        services: vec![],
        tmux_session: "sess".into(),
        created_at: "now".into(),
    };

    par_core::tmux::ensure_session(&s).unwrap();
    par_core::tmux::new_window(&s.tmux_session, "w1", &s.repo_path).unwrap();
    par_core::tmux::select_layout(&s.tmux_session, "tiled").unwrap();
    // attach too
    par_core::tmux::attach(&s.tmux_session).unwrap();

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("has-session"));
    assert!(logged.contains("new-session"));
    assert!(logged.contains("new-window"));
    assert!(logged.contains("select-layout"));
    assert!(logged.contains("attach-session"));
}
