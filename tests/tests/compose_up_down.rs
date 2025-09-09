use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

fn prepend_path(dir: &PathBuf) -> String {
    let old = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

#[test]
fn compose_up_down_uses_repo_files_and_docker() {
    let data_td = TempDir::new().unwrap();
    par_core::set_data_dir_override_for_testing(data_td.path());

    // Prepare repo with .belljar/compose file
    let repo_td = TempDir::new().unwrap();
    let repo = repo_td.path();
    let bj = repo.join(".belljar/compose");
    fs::create_dir_all(&bj).unwrap();
    let compose_file = bj.join("svc.yml");
    fs::write(&compose_file, "services: {}\n").unwrap();

    // Create docker shim
    let shim_dir = TempDir::new().unwrap();
    let log = shim_dir.path().join("docker.log");
    let shim = shim_dir.path().join("docker");
    let script = format!("#!/usr/bin/env bash\necho \"$@\" >> {}\nexit 0\n", log.display());
    fs::write(&shim, script).unwrap();
    let mut perm = fs::metadata(&shim).unwrap().permissions();
    perm.set_mode(0o755);
    fs::set_permissions(&shim, perm).unwrap();
    std::env::set_var("PATH", prepend_path(&shim_dir.path().to_path_buf()));

    // Create a session
    let s = par_core::create_session("t", repo, None, vec![]).unwrap();
    // Up should succeed and call docker compose with -f pointing to our file
    par_core::compose::up(&s).unwrap();
    par_core::compose::down(&s).unwrap();

    let logged = fs::read_to_string(&log).unwrap();
    assert!(logged.contains("compose"));
    assert!(logged.contains("-p"));
    assert!(logged.contains("-f"));
    assert!(logged.contains(compose_file.file_name().unwrap().to_string_lossy().as_ref()));
    assert!(logged.contains("up -d"));
    assert!(logged.contains("down -v"));
}

