use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn prepend_path(dir: &std::path::Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

fn make_tmux_shim() -> TempDir {
    let dir = TempDir::new().unwrap();
    let shim = dir.path().join("tmux");
    // fail new-window and select-layout
    fs::write(&shim, "#!/usr/bin/env bash\nif [ \"$1\" = new-window ]; then exit 1; fi\nif [ \"$1\" = select-layout ]; then exit 1; fi\nexit 0\n").unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    dir
}

#[test]
fn new_window_and_select_layout_errors() {
    let shim_dir = make_tmux_shim();
    std::env::set_var("PATH", prepend_path(shim_dir.path()));

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

    // new_window should error
    let e = par_core::tmux::new_window(&s.tmux_session, "w1", &s.repo_path).unwrap_err();
    match e { par_core::CoreError::Tmux(_) => {}, _ => panic!("unexpected error") }
    // select_layout should error
    let e = par_core::tmux::select_layout(&s.tmux_session, "tiled").unwrap_err();
    match e { par_core::CoreError::Tmux(_) => {}, _ => panic!("unexpected error") }
}

