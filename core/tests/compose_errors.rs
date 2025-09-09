use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn prepend_path(dir: &std::path::Path) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn compose_up_error() {
    let data = TempDir::new().unwrap();
    par_core::set_data_dir_override_for_testing(data.path());
    let repo = TempDir::new().unwrap();
    // root compose present
    fs::write(repo.path().join("docker-compose.yml"), "version: '3.9'\nservices: {}\n").unwrap();

    // docker shim fails
    let shim_dir = TempDir::new().unwrap();
    let shim = shim_dir.path().join("docker");
    fs::write(&shim, "#!/usr/bin/env bash\nexit 1\n").unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    std::env::set_var("PATH", prepend_path(shim_dir.path()));

    let s = par_core::create_session("t", repo.path(), None, vec![]).unwrap();
    match par_core::compose::up(&s) { Err(par_core::CoreError::Compose(_)) => {}, _ => panic!("expected compose error") }
}

#[test]
fn compose_down_error() {
    let data = TempDir::new().unwrap();
    par_core::set_data_dir_override_for_testing(data.path());
    let repo = TempDir::new().unwrap();
    // root compose present
    fs::write(repo.path().join("docker-compose.yml"), "version: '3.9'\nservices: {}\n").unwrap();

    // docker shim fails
    let shim_dir = TempDir::new().unwrap();
    let shim = shim_dir.path().join("docker");
    fs::write(&shim, "#!/usr/bin/env bash\nexit 1\n").unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    std::env::set_var("PATH", prepend_path(shim_dir.path()));

    let s = par_core::create_session("t", repo.path(), None, vec![]).unwrap();
    match par_core::compose::down(&s) { Err(par_core::CoreError::Compose(_)) => {}, _ => panic!("expected compose error") }
}
