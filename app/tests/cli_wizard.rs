use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn wizard_creates_dockerfiles_rust_codex() {
    let td = TempDir::new().unwrap();

    let bin = env!("CARGO_BIN_EXE_belljar");
    let mut child = Command::new(bin)
        .current_dir(td.path())
        .arg("wizard")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"1\n1\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let df = td.path().join("Dockerfile");
    let ai = td.path().join("Dockerfile.ai");
    assert!(df.exists(), "Dockerfile should be created");
    assert!(ai.exists(), "Dockerfile.ai should be created");

    let df_s = fs::read_to_string(&df).unwrap();
    let ai_s = fs::read_to_string(&ai).unwrap();
    assert!(df_s.contains("FROM rust:"));
    assert!(ai_s.to_lowercase().contains("openai"));
}

#[test]
fn wizard_creates_dockerfiles_python_aider() {
    let td = TempDir::new().unwrap();

    let bin = env!("CARGO_BIN_EXE_belljar");
    let mut child = Command::new(bin)
        .current_dir(td.path())
        .arg("wizard")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"2\n4\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());

    let df = td.path().join("Dockerfile");
    let ai = td.path().join("Dockerfile.ai");
    assert!(df.exists());
    assert!(ai.exists());
    let df_s = fs::read_to_string(&df).unwrap();
    let ai_s = fs::read_to_string(&ai).unwrap();
    assert!(df_s.contains("FROM python:3.11"));
    assert!(ai_s.to_lowercase().contains("aider"));
}

#[test]
fn wizard_respects_overwrite_prompt() {
    let td = TempDir::new().unwrap();
    // First run to create files
    {
        let bin = env!("CARGO_BIN_EXE_belljar");
        let mut child = Command::new(bin)
            .current_dir(td.path())
            .arg("wizard")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.as_mut().unwrap().write_all(b"1\n1\n").unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
    }

    // Second run, choose Python + Goose but answer 'n' to overwrite so original stays
    let printed = {
        let bin = env!("CARGO_BIN_EXE_belljar");
        let mut child = Command::new(bin)
            .current_dir(td.path())
            .arg("wizard")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"2\n3\nn\nn\n")
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    assert!(printed.contains("Skipped"));

    // Content remains from Rust/Codex
    let df_s = fs::read_to_string(td.path().join("Dockerfile")).unwrap();
    let ai_s = fs::read_to_string(td.path().join("Dockerfile.ai")).unwrap();
    assert!(df_s.contains("FROM rust:"));
    assert!(ai_s.to_lowercase().contains("openai"));
}
