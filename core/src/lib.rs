//! par-core: internal library for par-rs

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

/// Returns the semantic version of the core crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("registry path not found")]
    NoRegistryPath,
    #[error("compose error: {0}")]
    Compose(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub label: String,
    pub repo_path: PathBuf,
    pub branch: Option<String>,
    pub worktree_path: Option<PathBuf>,
    pub compose_project: String,
    pub services: Vec<String>,
    pub tmux_session: String,
    pub created_at: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub sessions: Vec<Session>,
}

fn data_dir() -> Result<PathBuf, CoreError> {
    let dirs = ProjectDirs::from("dev", "par-rs", "par-rs").ok_or(CoreError::NoRegistryPath)?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn registry_path() -> Result<PathBuf, CoreError> {
    Ok(data_dir()?.join("registry.json"))
}

pub fn load_registry() -> Result<Registry, CoreError> {
    let path = registry_path()?;
    if !path.exists() {
        return Ok(Registry::default());
    }
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let reg: Registry = serde_json::from_str(&s)?;
    Ok(reg)
}

pub fn save_registry(reg: &Registry) -> Result<(), CoreError> {
    let path = registry_path()?;
    let mut f = File::create(path)?;
    let s = serde_json::to_string_pretty(reg)?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

pub fn create_session(label: &str, repo_path: &Path, branch: Option<String>, services: Vec<String>) -> Result<Session, CoreError> {
    let mut reg = load_registry()?;
    let id = Uuid::new_v4().to_string();
    let compose_project = format!("parrs_{}", &id[..8]);
    let tmux_session = label.to_string();
    let created_at = OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
    let session = Session {
        id,
        label: label.to_string(),
        repo_path: repo_path.to_path_buf(),
        branch,
        worktree_path: None,
        compose_project,
        services,
        tmux_session,
        created_at,
    };
    reg.sessions.push(session.clone());
    save_registry(&reg)?;
    Ok(session)
}

pub fn remove_session(label_or_id: &str) -> Result<Option<Session>, CoreError> {
    let mut reg = load_registry()?;
    if let Some(idx) = reg
        .sessions
        .iter()
        .position(|s| s.label == label_or_id || s.id == label_or_id)
    {
        let s = reg.sessions.remove(idx);
        save_registry(&reg)?;
        return Ok(Some(s));
    }
    Ok(None)
}

pub fn find_session(label_or_id: &str) -> Result<Option<Session>, CoreError> {
    let reg = load_registry()?;
    Ok(reg
        .sessions
        .into_iter()
        .find(|s| s.label == label_or_id || s.id == label_or_id))
}

pub mod compose {
    use super::{data_dir, CoreError, Session};
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;

    const BASE_YML: &str = r#"version: '3.9'
services: {}
"#;

    const POSTGRES_YML: &str = r#"services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_USER: postgres
      POSTGRES_DB: app
    ports:
      - "5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5
"#;

    const REDIS_YML: &str = r#"services:
  redis:
    image: redis:7
    ports:
      - "6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5
"#;

    pub struct ComposeFiles {
        pub dir: PathBuf,
        pub files: Vec<PathBuf>,
    }

    pub fn session_compose_dir(session: &Session) -> Result<PathBuf, CoreError> {
        let dir = data_dir()?.join("sessions").join(&session.id).join("compose");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn write_files(session: &Session) -> Result<ComposeFiles, CoreError> {
        let dir = session_compose_dir(session)?;
        let mut files = Vec::new();

        let base = dir.join("base.yml");
        fs::write(&base, BASE_YML)?;
        files.push(base);

        for svc in &session.services {
            let (name, contents) = match svc.as_str() {
                "postgres" => ("postgres.yml", POSTGRES_YML),
                "redis" => ("redis.yml", REDIS_YML),
                other => {
                    let mut f = fs::File::create(dir.join(format!("{}.yml", other)))?;
                    f.write_all(b"services: {}\n")?;
                    files.push(dir.join(format!("{}.yml", other)));
                    continue;
                }
            };
            let path = dir.join(name);
            fs::write(&path, contents)?;
            files.push(path);
        }
        Ok(ComposeFiles { dir, files })
    }

    pub fn up(session: &Session) -> Result<(), CoreError> {
        let cf = write_files(session)?;
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("-p").arg(&session.compose_project);
        for f in &cf.files {
            cmd.arg("-f").arg(f);
        }
        cmd.arg("up").arg("-d");
        let status = cmd.status().map_err(|e| CoreError::Compose(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Compose(format!(
                "docker compose up failed with status {}",
                status
            )));
        }
        Ok(())
    }

    pub fn down(session: &Session) -> Result<(), CoreError> {
        let cf_dir = session_compose_dir(session)?;
        // Reconstruct file list: base + known service files if present
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("-p").arg(&session.compose_project);
        let candidates = ["base.yml", "postgres.yml", "redis.yml"];
        for c in candidates {
            let p = cf_dir.join(c);
            if p.exists() {
                cmd.arg("-f").arg(p);
            }
        }
        cmd.arg("down").arg("-v");
        let status = cmd.status().map_err(|e| CoreError::Compose(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Compose(format!(
                "docker compose down failed with status {}",
                status
            )));
        }
        Ok(())
    }
}
