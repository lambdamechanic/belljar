use tempfile::TempDir;

#[test]
fn compose_discovery_falls_back_to_root() {
    let td = TempDir::new().unwrap();
    let repo = td.path();
    // root compose present
    std::fs::write(
        repo.join("docker-compose.yaml"),
        "version: '3.9'\nservices: {}\n",
    )
    .unwrap();
    // .belljar/compose exists but is empty
    std::fs::create_dir_all(repo.join(".belljar/compose")).unwrap();

    let files = par_core::compose::discover_files_for_repo(repo);
    let names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, vec!["docker-compose.yaml"]);
}
