use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

static PATH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn prepend_path(dir: &std::path::Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn compose_up_error() {
    if std::env::var("DOCKER_TESTS").is_err() {
        eprintln!("skipping compose_up_error: DOCKER_TESTS unset");
        return;
    }

    let _guard = PATH_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let orig_path = std::env::var("PATH").unwrap_or_default();

    let data = TempDir::new().unwrap();
    belljar_core::set_data_dir_override_for_testing(data.path());
    let repo = TempDir::new().unwrap();
    // root compose present
    fs::write(
        repo.path().join("docker-compose.yml"),
        "version: '3.9'\nservices: {}\n",
    )
    .unwrap();

    // create session with real docker
    let s = belljar_core::create_session("t", repo.path(), None, vec![]).unwrap();

    // docker shim fails
    let shim_dir = TempDir::new().unwrap();
    let shim = shim_dir.path().join("docker");
    fs::write(&shim, "#!/usr/bin/env bash\nexit 1\n").unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    std::env::set_var("PATH", prepend_path(shim_dir.path()));

    match belljar_core::compose::up(&s) {
        Err(belljar_core::CoreError::Compose(_)) => {}
        _ => panic!("expected compose error"),
    }

    std::env::set_var("PATH", orig_path);
}

#[test]
fn compose_down_error() {
    if std::env::var("DOCKER_TESTS").is_err() {
        eprintln!("skipping compose_down_error: DOCKER_TESTS unset");
        return;
    }

    let _guard = PATH_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let orig_path = std::env::var("PATH").unwrap_or_default();

    let data = TempDir::new().unwrap();
    belljar_core::set_data_dir_override_for_testing(data.path());
    let repo = TempDir::new().unwrap();
    // root compose present
    fs::write(
        repo.path().join("docker-compose.yml"),
        "version: '3.9'\nservices: {}\n",
    )
    .unwrap();

    // create session with real docker
    let s = belljar_core::create_session("t", repo.path(), None, vec![]).unwrap();

    // docker shim fails
    let shim_dir = TempDir::new().unwrap();
    let shim = shim_dir.path().join("docker");
    fs::write(&shim, "#!/usr/bin/env bash\nexit 1\n").unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    std::env::set_var("PATH", prepend_path(shim_dir.path()));

    match belljar_core::compose::down(&s) {
        Err(belljar_core::CoreError::Compose(_)) => {}
        _ => panic!("expected compose error"),
    }

    std::env::set_var("PATH", orig_path);
}
