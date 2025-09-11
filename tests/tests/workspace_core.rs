use tempfile::TempDir;

fn init_data() -> TempDir {
    let td = TempDir::new().unwrap();
    par_core::set_data_dir_override_for_testing(td.path());
    td
}

#[test]
fn workspace_crud_roundtrip() {
    let _data = init_data();
    // list empty
    assert!(par_core::list_workspaces().unwrap().is_empty());

    let root = TempDir::new().unwrap();
    let repos = vec![root.path().join("frontend"), root.path().join("backend")];
    std::fs::create_dir_all(&repos[0]).unwrap();
    std::fs::create_dir_all(&repos[1]).unwrap();

    // create
    let ws = par_core::create_workspace("dev-ws", root.path(), repos.clone()).unwrap();
    assert_eq!(ws.label, "dev-ws");
    assert!(ws.tmux_session.starts_with("ws-"));

    // list -> 1
    let list = par_core::list_workspaces().unwrap();
    assert_eq!(list.len(), 1);

    // find by label
    let found = par_core::find_workspace("dev-ws").unwrap();
    assert!(found.is_some());

    // remove
    let removed = par_core::remove_workspace("dev-ws").unwrap();
    assert!(removed.is_some());
    assert!(par_core::list_workspaces().unwrap().is_empty());
}
