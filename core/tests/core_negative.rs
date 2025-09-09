use tempfile::TempDir;

#[test]
fn compose_up_down_no_files() {
    let data = TempDir::new().unwrap();
    par_core::set_data_dir_override_for_testing(data.path());
    let repo = TempDir::new().unwrap();
    let s = par_core::create_session("t", repo.path(), None, vec![]).unwrap();
    let up = par_core::compose::up(&s).unwrap_err();
    match up { par_core::CoreError::NoComposeFiles => {}, _ => panic!("unexpected error") }
    let down = par_core::compose::down(&s).unwrap_err();
    match down { par_core::CoreError::NoComposeFiles => {}, _ => panic!("unexpected error") }
}

#[test]
fn tmux_not_found() {
    // Temporarily clear PATH so which("tmux") fails
    let old = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", "/nonexistent");
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
    let e = par_core::tmux::ensure_session(&s).unwrap_err();
    match e { par_core::CoreError::TmuxNotFound => {}, _ => panic!("unexpected error: {e}") }
    std::env::set_var("PATH", old);
}

