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

