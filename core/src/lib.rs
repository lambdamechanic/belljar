//! par-core: internal library for par-rs

use directories::ProjectDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
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
    #[error("no compose files found in repository")]
    NoComposeFiles,
    #[error("tmux not found in PATH")]
    TmuxNotFound,
    #[error("tmux error: {0}")]
    Tmux(String),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub label: String,
    pub root_path: PathBuf,
    pub repos: Vec<PathBuf>,
    pub tmux_session: String,
    pub created_at: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub sessions: Vec<Session>,
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

static DATA_DIR_OVERRIDE: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

fn data_dir() -> Result<PathBuf, CoreError> {
    if let Ok(env_path) = std::env::var("BELLJAR_DATA_DIR") {
        let p = PathBuf::from(env_path);
        if !p.as_os_str().is_empty() {
            fs::create_dir_all(&p)?;
            return Ok(p);
        }
    }
    if let Some(p) = DATA_DIR_OVERRIDE.lock().unwrap().clone() {
        fs::create_dir_all(&p)?;
        return Ok(p);
    }
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

#[cfg(feature = "testing")]
pub fn set_data_dir_override_for_testing<P: Into<PathBuf>>(p: P) {
    *DATA_DIR_OVERRIDE.lock().unwrap() = Some(p.into());
}

#[cfg(feature = "testing")]
pub fn clear_data_dir_override_for_testing() {
    *DATA_DIR_OVERRIDE.lock().unwrap() = None;
}

pub fn create_session(
    label: &str,
    repo_path: &Path,
    branch: Option<String>,
    services: Vec<String>,
) -> Result<Session, CoreError> {
    let mut reg = load_registry()?;
    let id = Uuid::new_v4().to_string();
    let compose_project = format!("parrs_{}", &id[..8]);
    let tmux_session = label.to_string();
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
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

pub mod git {
    use super::{CoreError, Session};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn git() -> &'static str {
        "git"
    }

    pub fn is_git_repo(path: &Path) -> bool {
        Command::new(git())
            .arg("-C")
            .arg(path)
            .arg("rev-parse")
            .arg("--git-dir")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn ensure_worktree(
        repo: &Path,
        label: &str,
        branch: &Option<String>,
    ) -> Result<PathBuf, CoreError> {
        let wt_dir = repo.join(".belljar").join("worktrees").join(label);
        if let Some(parent) = wt_dir.parent() {
            fs::create_dir_all(parent)?;
        }

        let run = |args: &[&str]| -> Result<bool, CoreError> {
            let st = Command::new(git())
                .args(["-C"])
                .arg(repo)
                .args(args)
                .status()
                .map_err(|e| CoreError::Compose(format!("git failed: {e}")))?;
            Ok(st.success())
        };

        let ok = match branch {
            Some(br) => {
                // Try creating a new branch in the worktree, else attach existing branch
                run(&[
                    "worktree",
                    "add",
                    "-b",
                    br,
                    wt_dir.to_str().unwrap(),
                    "HEAD",
                ])
                .unwrap_or(false)
                    || run(&["worktree", "add", wt_dir.to_str().unwrap(), br]).unwrap_or(false)
            }
            None => run(&["worktree", "add", wt_dir.to_str().unwrap()]).unwrap_or(false),
        };

        if !ok && !wt_dir.exists() {
            return Err(CoreError::Compose("git worktree add failed".into()));
        }
        Ok(wt_dir)
    }

    pub fn set_session_worktree(session: &mut Session, path: PathBuf) -> Result<(), CoreError> {
        session.worktree_path = Some(path);
        // Persist the change
        let mut reg = super::load_registry()?;
        if let Some(s) = reg.sessions.iter_mut().find(|s| s.id == session.id) {
            s.worktree_path = session.worktree_path.clone();
        }
        super::save_registry(&reg)?;
        Ok(())
    }
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
    use super::{CoreError, Session};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn discover_files(repo_path: &Path) -> Result<Vec<PathBuf>, CoreError> {
        let mut files: Vec<PathBuf> = Vec::new();

        // Prefer per-repo belljar-specific directory if present
        let bj_dir = repo_path.join(".belljar").join("compose");
        if bj_dir.is_dir() {
            let mut entries: Vec<PathBuf> = fs::read_dir(&bj_dir)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    if let Some(ext) = p.extension() {
                        ext == "yml" || ext == "yaml"
                    } else {
                        false
                    }
                })
                .collect();
            entries.sort();
            if !entries.is_empty() {
                return Ok(entries);
            }
        }

        // Fallback to common compose filenames in repo root
        for name in [
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ] {
            let p = repo_path.join(name);
            if p.exists() {
                files.push(p);
            }
        }

        Ok(files)
    }

    #[cfg(feature = "testing")]
    pub fn discover_files_for_repo(repo_path: &Path) -> Vec<PathBuf> {
        discover_files(repo_path).unwrap_or_default()
    }

    pub fn up(session: &Session) -> Result<(), CoreError> {
        let files = discover_files(&session.repo_path)?;
        if files.is_empty() {
            return Err(CoreError::NoComposeFiles);
        }
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("-p").arg(&session.compose_project);
        for f in &files {
            cmd.arg("-f").arg(f);
        }
        cmd.arg("up").arg("-d");
        let status = cmd
            .status()
            .map_err(|e| CoreError::Compose(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Compose(format!(
                "docker compose up failed with status {status}"
            )));
        }
        Ok(())
    }

    pub fn down(session: &Session) -> Result<(), CoreError> {
        let files = discover_files(&session.repo_path)?;
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("-p").arg(&session.compose_project);
        if files.is_empty() {
            return Err(CoreError::NoComposeFiles);
        }
        for p in files {
            cmd.arg("-f").arg(p);
        }
        cmd.arg("down").arg("-v");
        let status = cmd
            .status()
            .map_err(|e| CoreError::Compose(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Compose(format!(
                "docker compose down failed with status {status}"
            )));
        }
        Ok(())
    }
}

pub fn create_workspace(
    label: &str,
    root: &Path,
    repos: Vec<PathBuf>,
) -> Result<Workspace, CoreError> {
    let mut reg = load_registry()?;
    let id = Uuid::new_v4().to_string();
    let tmux_session = format!("ws-{label}");
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    let ws = Workspace {
        id,
        label: label.to_string(),
        root_path: root.to_path_buf(),
        repos,
        tmux_session,
        created_at,
    };
    reg.workspaces.push(ws.clone());
    save_registry(&reg)?;
    Ok(ws)
}

pub fn list_workspaces() -> Result<Vec<Workspace>, CoreError> {
    Ok(load_registry()?.workspaces)
}

pub fn find_workspace(label_or_id: &str) -> Result<Option<Workspace>, CoreError> {
    let reg = load_registry()?;
    Ok(reg
        .workspaces
        .into_iter()
        .find(|w| w.label == label_or_id || w.id == label_or_id))
}

pub fn remove_workspace(label_or_id: &str) -> Result<Option<Workspace>, CoreError> {
    let mut reg = load_registry()?;
    if let Some(idx) = reg
        .workspaces
        .iter()
        .position(|w| w.label == label_or_id || w.id == label_or_id)
    {
        let w = reg.workspaces.remove(idx);
        save_registry(&reg)?;
        return Ok(Some(w));
    }
    Ok(None)
}

pub mod tmux {
    use super::{CoreError, Session};
    use std::path::Path;
    use std::process::Command;
    use which::which;

    fn tmux_bin() -> Result<String, CoreError> {
        which("tmux")
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(|_| CoreError::TmuxNotFound)
    }

    pub fn has_session(name: &str) -> Result<bool, CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .args(["has-session", "-t", name])
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        Ok(status.success())
    }

    pub fn new_detached(name: &str, cwd: &Path) -> Result<(), CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .args(["new-session", "-d", "-s", name, "-c"])
            .arg(cwd)
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Tmux("failed to create session".into()));
        }
        Ok(())
    }

    pub fn attach(name: &str) -> Result<(), CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .args(["attach-session", "-t", name])
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Tmux("failed to attach".into()));
        }
        Ok(())
    }

    pub fn ensure_session(session: &Session) -> Result<(), CoreError> {
        if !has_session(&session.tmux_session)? {
            new_detached(&session.tmux_session, &session.repo_path)?;
        }
        // Best-effort tagging so users can style/status belljar sessions in tmux.
        let _ = tag_belljar_session(&session.tmux_session);
        Ok(())
    }

    pub fn send_keys(name: &str, command: &str) -> Result<(), CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .arg("send-keys")
            .arg("-t")
            .arg(name)
            .arg(command)
            .arg("C-m")
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Tmux("failed to send keys".into()));
        }
        Ok(())
    }

    pub fn ensure_named_session(name: &str, cwd: &Path) -> Result<(), CoreError> {
        if !has_session(name)? {
            new_detached(name, cwd)?;
        }
        // Best-effort tagging so users can style/status belljar sessions in tmux.
        let _ = tag_belljar_session(name);
        Ok(())
    }

    pub fn new_window(session_name: &str, window_name: &str, cwd: &Path) -> Result<(), CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .args(["new-window", "-t", session_name, "-n", window_name, "-c"])
            .arg(cwd)
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Tmux("failed to create window".into()));
        }
        Ok(())
    }

    pub fn select_layout(session_name: &str, layout: &str) -> Result<(), CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .args(["select-layout", "-t", session_name, layout])
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Tmux("failed to select layout".into()));
        }
        Ok(())
    }

    fn tag_belljar_session(name: &str) -> Result<(), CoreError> {
        let tmux = tmux_bin()?;
        let status = Command::new(tmux)
            .args(["set-option", "-t", name, "@belljar", "1"])
            .status()
            .map_err(|e| CoreError::Tmux(e.to_string()))?;
        if !status.success() {
            return Err(CoreError::Tmux("failed to tag session".into()));
        }
        Ok(())
    }
}
