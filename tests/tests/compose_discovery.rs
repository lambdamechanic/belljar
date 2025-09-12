use tempfile::TempDir;

#[test]
fn compose_discovery_prefers_belljar_dir() {
    let td = TempDir::new().unwrap();
    let repo = td.path();
    // root compose present
    std::fs::write(
        repo.join("docker-compose.yml"),
        "version: '3.9'\nservices: {}\n",
    )
    .unwrap();
    // .belljar/compose present with two files
    let bj_dir = repo.join(".belljar/compose");
    std::fs::create_dir_all(&bj_dir).unwrap();
    std::fs::write(bj_dir.join("a.yml"), "services: {}\n").unwrap();
    std::fs::write(bj_dir.join("b.yaml"), "services: {}\n").unwrap();

    let files = belljar_core::compose::discover_files_for_repo(repo);
    // Should include only the two files from .belljar/compose, sorted
    let mut names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["a.yml", "b.yaml"]);
}
