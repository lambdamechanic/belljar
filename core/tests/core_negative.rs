use tempfile::TempDir;

#[test]
fn compose_up_down_no_files() {
    let data = TempDir::new().unwrap();
    belljar_core::set_data_dir_override_for_testing(data.path());
    let repo = TempDir::new().unwrap();
    let s = belljar_core::create_session("t", repo.path(), None, vec![]).unwrap();
    let up = belljar_core::compose::up(&s).unwrap_err();
    match up {
        belljar_core::CoreError::NoComposeFiles => {}
        _ => panic!("unexpected error"),
    }
    let down = belljar_core::compose::down(&s).unwrap_err();
    match down {
        belljar_core::CoreError::NoComposeFiles => {}
        _ => panic!("unexpected error"),
    }
}

#[test]
fn tmux_not_found() {
    // Temporarily clear PATH so which("tmux") fails
    let old = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", "/nonexistent");
    let s = belljar_core::Session {
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
    let e = belljar_core::tmux::ensure_session(&s).unwrap_err();
    match e {
        belljar_core::CoreError::TmuxNotFound => {}
        _ => panic!("unexpected error: {e}"),
    }
    std::env::set_var("PATH", old);
}
