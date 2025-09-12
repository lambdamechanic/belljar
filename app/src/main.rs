use clap::{Args, Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "belljar",
    version,
    about = "belljar: session/worktree manager with per-session Docker Compose isolation"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new session with a worktree and optional services
    Start(StartArgs),
    /// Checkout an existing branch/PR into a session
    Checkout {
        target: String,
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(long)]
        label: Option<String>,
    },
    /// List all sessions/workspaces
    Ls,
    /// Open a session in tmux
    Open { label: String },
    /// Remove a session or all sessions
    Rm { target: String },
    /// Send a command to a session or all
    Send {
        target: String,
        command: Vec<String>,
    },
    /// Show control center (placeholder)
    ControlCenter,
    /// Workspace subcommands
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCmd,
    },
    /// Print internal version details
    Version,
    /// Run an interactive setup wizard to scaffold Dockerfiles
    Wizard,
}

#[derive(Args, Debug)]
struct StartArgs {
    label: String,
    /// Path to git repository
    #[arg(short, long)]
    path: Option<PathBuf>,
    /// Branch name to create/checkout
    #[arg(long)]
    branch: Option<String>,
}

#[derive(Subcommand, Debug)]
enum WorkspaceCmd {
    /// List all workspaces
    Ls,
    /// Create a workspace with optional repos
    Start {
        label: String,
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Comma-separated relative repo paths under root
        #[arg(long, value_delimiter = ',')]
        repos: Vec<String>,
        /// Open in tmux after creating
        #[arg(long)]
        open: bool,
    },
    /// Open a workspace tmux session
    Open { label: String },
    /// Remove a workspace by label or id
    Rm { target: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start(args) => {
            let repo = resolve_repo_path(args.path.as_deref())?;
            let mut session =
                belljar_core::create_session(&args.label, &repo, args.branch.clone(), vec![])
                    .map_err(|e| anyhow::anyhow!("create session failed: {e}"))?;

            // Ensure worktree if repo is git
            if belljar_core::git::is_git_repo(&repo) {
                match belljar_core::git::ensure_worktree(&repo, &args.label, &args.branch) {
                    Ok(wt) => {
                        belljar_core::git::set_session_worktree(&mut session, wt).ok();
                    }
                    Err(e) => eprintln!("warning: worktree setup failed: {e}"),
                }
            }
            match belljar_core::compose::up(&session) {
                Ok(()) => {
                    println!(
                        "created session: {} (project: {}) [compose up]",
                        session.label, session.compose_project
                    );
                }
                Err(belljar_core::CoreError::NoComposeFiles) => {
                    println!(
                        "created session: {} (project: {}); no compose files found, skipping",
                        session.label, session.compose_project
                    );
                }
                Err(e) => {
                    eprintln!("warning: compose up failed: {e}");
                    println!(
                        "created session: {} (project: {})",
                        session.label, session.compose_project
                    );
                }
            }
        }
        Commands::Checkout {
            target,
            path,
            label,
        } => {
            let repo = resolve_repo_path(path.as_deref())?;
            let label = label.unwrap_or_else(|| target.clone());
            let mut session = belljar_core::create_session(&label, &repo, Some(target.clone()), vec![])
                .map_err(|e| anyhow::anyhow!("create session failed: {e}"))?;
            if belljar_core::git::is_git_repo(&repo) {
                match belljar_core::git::ensure_worktree(&repo, &label, &Some(target.clone())) {
                    Ok(wt) => {
                        belljar_core::git::set_session_worktree(&mut session, wt).ok();
                    }
                    Err(e) => eprintln!("warning: worktree setup failed: {e}"),
                }
            }
            match belljar_core::compose::up(&session) {
                Ok(()) => println!(
                    "checked out: {} -> {} (project: {}) [compose up]",
                    label, target, session.compose_project
                ),
                Err(belljar_core::CoreError::NoComposeFiles) => println!(
                    "checked out: {} -> {} (project: {}); no compose files found, skipping",
                    label, target, session.compose_project
                ),
                Err(e) => eprintln!("warning: compose up failed: {e}"),
            }
        }
        Commands::Ls => {
            let reg = belljar_core::load_registry()
                .map_err(|e| anyhow::anyhow!("load registry failed: {e}"))?;
            if reg.sessions.is_empty() {
                println!("no sessions");
            } else {
                for s in reg.sessions {
                    println!(
                        "{}\t{}\t{}",
                        s.label,
                        s.repo_path.display(),
                        s.compose_project
                    );
                }
            }
        }
        Commands::Open { label } => {
            match belljar_core::find_session(&label) {
                Ok(Some(s)) => {
                    match belljar_core::tmux::ensure_session(&s) {
                        Ok(()) => {
                            // Try to attach. If tmux not found, provide guidance.
                            match belljar_core::tmux::attach(&s.tmux_session) {
                                Ok(()) => {}
                                Err(belljar_core::CoreError::TmuxNotFound) => {
                                    println!(
                                        "tmux not found; cd {} to work in this session",
                                        s.repo_path.display()
                                    );
                                }
                                Err(e) => eprintln!("failed to attach: {e}"),
                            }
                        }
                        Err(belljar_core::CoreError::TmuxNotFound) => {
                            println!(
                                "tmux not found; cd {} to work in this session",
                                s.repo_path.display()
                            );
                        }
                        Err(e) => eprintln!("failed to open session: {e}"),
                    }
                }
                Ok(None) => println!("no such session: {label}"),
                Err(e) => eprintln!("failed to load registry: {e}"),
            }
        }
        Commands::Rm { target } => {
            if target == "all" {
                let reg = belljar_core::load_registry()
                    .map_err(|e| anyhow::anyhow!("load registry failed: {e}"))?;
                for s in reg.sessions {
                    match belljar_core::compose::down(&s) {
                        Ok(()) | Err(belljar_core::CoreError::NoComposeFiles) => {}
                        Err(e) => eprintln!("warning: compose down failed for {}: {e}", s.label),
                    }
                    let _ = belljar_core::remove_session(&s.label);
                    println!("removed {}", s.label);
                }
            } else if let Some(s) = belljar_core::find_session(&target)
                .map_err(|e| anyhow::anyhow!("find session failed: {e}"))?
            {
                match belljar_core::compose::down(&s) {
                    Ok(()) | Err(belljar_core::CoreError::NoComposeFiles) => {}
                    Err(e) => eprintln!("warning: compose down failed for {}: {e}", s.label),
                }
                belljar_core::remove_session(&target)
                    .map_err(|e| anyhow::anyhow!("remove failed: {e}"))?;
                println!("removed {}", s.label);
            } else {
                println!("no such session: {target}");
            }
        }
        Commands::Send { target, command } => {
            let cmd = command.join(" ");
            if target == "all" {
                match belljar_core::load_registry() {
                    Ok(reg) => {
                        for s in reg.sessions {
                            match belljar_core::tmux::ensure_session(&s) {
                                Ok(()) => {
                                    if let Err(e) = belljar_core::tmux::send_keys(&s.tmux_session, &cmd)
                                    {
                                        eprintln!("send to {} failed: {e}", s.label);
                                    }
                                }
                                Err(e) => eprintln!("ensure session {} failed: {e}", s.label),
                            }
                        }
                    }
                    Err(e) => eprintln!("failed to load registry: {e}"),
                }
            } else {
                match belljar_core::find_session(&target) {
                    Ok(Some(s)) => match belljar_core::tmux::ensure_session(&s) {
                        Ok(()) => {
                            if let Err(e) = belljar_core::tmux::send_keys(&s.tmux_session, &cmd) {
                                eprintln!("send failed: {e}");
                            }
                        }
                        Err(e) => eprintln!("ensure session failed: {e}"),
                    },
                    Ok(None) => println!("no such session: {target}"),
                    Err(e) => eprintln!("failed to load registry: {e}"),
                }
            }
        }
        Commands::ControlCenter => {
            // Create a tmux session named "belljar-cc" with one window per session
            let cc_name = "belljar-cc";
            match belljar_core::load_registry() {
                Ok(reg) => {
                    if reg.sessions.is_empty() {
                        println!("no sessions to show");
                    } else {
                        // Use the first session's repo as the base cwd
                        let base = &reg.sessions[0].repo_path;
                        match belljar_core::tmux::ensure_named_session(cc_name, base) {
                            Ok(()) => {
                                for s in reg.sessions {
                                    // Try to create a window per session label
                                    if let Err(e) =
                                        belljar_core::tmux::new_window(cc_name, &s.label, &s.repo_path)
                                    {
                                        eprintln!("failed to create window for {}: {e}", s.label);
                                    }
                                }
                                // Optional: choose a tiled layout
                                let _ = belljar_core::tmux::select_layout(cc_name, "tiled");
                                // Attach to control center
                                if let Err(e) = belljar_core::tmux::attach(cc_name) {
                                    eprintln!("failed to attach control center: {e}");
                                }
                            }
                            Err(belljar_core::CoreError::TmuxNotFound) => {
                                println!("tmux not found; control-center requires tmux");
                            }
                            Err(e) => eprintln!("failed to init control center: {e}"),
                        }
                    }
                }
                Err(e) => eprintln!("failed to load registry: {e}"),
            }
        }
        Commands::Workspace { command: ws } => match ws {
            WorkspaceCmd::Ls => match belljar_core::list_workspaces() {
                Ok(list) => {
                    if list.is_empty() {
                        println!("no workspaces");
                    } else {
                        for w in list {
                            println!("{}\t{}", w.label, w.root_path.display());
                        }
                    }
                }
                Err(e) => eprintln!("failed to list workspaces: {e}"),
            },
            WorkspaceCmd::Start {
                label,
                path,
                repos,
                open,
            } => {
                let root = resolve_repo_path(path.as_deref())?;
                let repo_paths: Vec<PathBuf> = repos.into_iter().map(|r| root.join(r)).collect();
                match belljar_core::create_workspace(&label, &root, repo_paths) {
                    Ok(ws) => {
                        println!("created workspace: {}", ws.label);
                        if open {
                            let _ = belljar_core::tmux::ensure_named_session(
                                &ws.tmux_session,
                                &ws.root_path,
                            );
                            let _ = belljar_core::tmux::attach(&ws.tmux_session);
                        }
                    }
                    Err(e) => eprintln!("failed to create workspace: {e}"),
                }
            }
            WorkspaceCmd::Open { label } => match belljar_core::find_workspace(&label) {
                Ok(Some(ws)) => {
                    if let Err(e) =
                        belljar_core::tmux::ensure_named_session(&ws.tmux_session, &ws.root_path)
                    {
                        eprintln!("failed to ensure workspace session: {e}");
                    }
                    for repo in &ws.repos {
                        let name = repo
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| repo.display().to_string());
                        if let Err(e) = belljar_core::tmux::new_window(&ws.tmux_session, &name, repo) {
                            eprintln!("failed to create window for {name}: {e}");
                        }
                    }
                    let _ = belljar_core::tmux::select_layout(&ws.tmux_session, "tiled");
                    if let Err(e) = belljar_core::tmux::attach(&ws.tmux_session) {
                        eprintln!("failed to attach workspace: {e}");
                    }
                }
                Ok(None) => println!("no such workspace: {label}"),
                Err(e) => eprintln!("failed to load registry: {e}"),
            },
            WorkspaceCmd::Rm { target } => match belljar_core::remove_workspace(&target) {
                Ok(Some(ws)) => println!("removed workspace {}", ws.label),
                Ok(None) => println!("no such workspace: {target}"),
                Err(e) => eprintln!("failed to remove workspace: {e}"),
            },
        },
        Commands::Version => {
            println!(
                "par {} (core {})",
                env!("CARGO_PKG_VERSION"),
                belljar_core::version()
            );
        }
        Commands::Wizard => {
            run_wizard()?;
        }
    }
    Ok(())
}

fn resolve_repo_path(path: Option<&Path>) -> anyhow::Result<PathBuf> {
    let p = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };
    if !p.exists() {
        anyhow::bail!("path does not exist: {}", p.display());
    }
    Ok(p)
}

#[derive(Debug, Clone, Copy)]
enum Language {
    Rust,
    Python,
}

#[derive(Debug, Clone, Copy)]
enum AiCoder {
    Codex,
    Claude,
    Goose,
    Aider,
}

fn run_wizard() -> anyhow::Result<()> {
    println!("belljar wizard — scaffold Dockerfiles for your project\n");
    let lang = prompt_language()?;
    let ai = prompt_ai()?;

    let cwd = std::env::current_dir()?;
    // For now, write scaffolded Dockerfiles at the project root to match expectations
    // in our CLI tests. Users can move them under .belljar/compose later if desired.
    println!("\nScaffolding in {}", cwd.display());

    // Language-specific Dockerfile for development
    let (lang_filename, lang_contents) = match lang {
        Language::Rust => ("Dockerfile.dev", rust_dockerfile_template()),
        Language::Python => ("Dockerfile.dev", python_dockerfile_template()),
    };

    // AI helper Dockerfile layered on top of the dev image
    let (ai_filename, ai_contents) = match ai {
        AiCoder::Codex => ("Dockerfile.ai", ai_codex_dockerfile_template()),
        AiCoder::Claude => ("Dockerfile.ai", ai_claude_dockerfile_template()),
        AiCoder::Goose => ("Dockerfile.ai", ai_goose_dockerfile_template()),
        AiCoder::Aider => ("Dockerfile.ai", ai_aider_dockerfile_template()),
    };

    write_with_prompt(&cwd.join(lang_filename), lang_contents.as_bytes())?;
    write_with_prompt(&cwd.join(ai_filename), ai_contents.as_bytes())?;

    println!("\nDone. Generated:");
    println!("  - {}", cwd.join(lang_filename).display());
    println!("  - {}", cwd.join(ai_filename).display());
    println!(
        "\nTip: add compose files under .belljar/compose/ to wire these into belljar sessions."
    );
    Ok(())
}

fn prompt_language() -> anyhow::Result<Language> {
    loop {
        println!("Choose language:");
        println!("  1) Rust");
        println!("  2) Python");
        print!("Enter choice [1/2]: ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let s = buf.trim().to_lowercase();
        match s.as_str() {
            "1" | "r" | "rust" => return Ok(Language::Rust),
            "2" | "p" | "py" | "python" => return Ok(Language::Python),
            _ => {
                println!("Invalid choice: {s}\n");
            }
        }
    }
}

fn prompt_ai() -> anyhow::Result<AiCoder> {
    loop {
        println!("Choose AI coder:");
        println!("  1) Codex");
        println!("  2) Claude");
        println!("  3) Goose");
        println!("  4) Aider");
        print!("Enter choice [1-4]: ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let s = buf.trim().to_lowercase();
        match s.as_str() {
            "1" | "codex" => return Ok(AiCoder::Codex),
            "2" | "claude" => return Ok(AiCoder::Claude),
            "3" | "goose" => return Ok(AiCoder::Goose),
            "4" | "aider" => return Ok(AiCoder::Aider),
            _ => println!("Invalid choice: {s}\n"),
        }
    }
}

fn write_with_prompt(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    if path.exists() {
        println!("{} already exists.", path.display());
        loop {
            print!("Overwrite? [y/N]: ");
            io::stdout().flush()?;
            let mut ans = String::new();
            io::stdin().read_line(&mut ans)?;
            let a = ans.trim().to_lowercase();
            match a.as_str() {
                "y" | "yes" => {
                    fs::write(path, contents)?;
                    println!("Overwrote {}", path.display());
                    break;
                }
                "n" | "no" | "" => {
                    println!("Skipped {}", path.display());
                    break;
                }
                _ => println!("Please answer 'y' or 'n'."),
            }
        }
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
        println!("Created {}", path.display());
    }
    Ok(())
}

fn rust_dockerfile_template() -> String {
    let t = r#"# syntax=docker/dockerfile:1
FROM rust:1-bookworm AS base

WORKDIR /workspace

# Common native deps (adjust as needed)
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential pkg-config libssl-dev ca-certificates curl git \
    && rm -rf /var/lib/apt/lists/*

# Optionally pre-build deps for faster iterative builds
# COPY Cargo.toml Cargo.lock ./
# RUN mkdir -p src && echo "fn main(){}" > src/main.rs \
#     && cargo build --release \
#     && rm -rf src

# Bring in your code
COPY . .

# Build command example:
# RUN cargo build --release

CMD ["bash"]
"#;
    t.to_string()
}

fn python_dockerfile_template() -> String {
    let t = r#"# syntax=docker/dockerfile:1
FROM python:3.11-slim AS base

ENV PIP_NO_CACHE_DIR=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

WORKDIR /workspace

# System deps (adjust as needed)
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential git curl \
    && rm -rf /var/lib/apt/lists/*

# Install app deps if present
COPY requirements*.txt ./
RUN if ls requirements*.txt >/dev/null 2>&1; then \
      pip install -r requirements.txt || true; \
    fi

# Bring in your code
COPY . .

CMD ["bash"]
"#;
    t.to_string()
}

fn ai_codex_dockerfile_template() -> String {
    // Codex helper container: installs @openai/codex and open-codex via npm on top of the dev image
    let t = r#"# syntax=docker/dockerfile:1.4
# Helper container for using OpenAI Codex workflows (Node)
FROM dockerfile:Dockerfile.dev

WORKDIR /workspace

# Install Node.js, npm, and the OpenAI Codex SDK plus open-codex
RUN apt-get update \
    && apt-get install -y --no-install-recommends nodejs npm \
    && npm install -g @openai/codex open-codex \
    && rm -rf /var/lib/apt/lists/*

# Set your credentials at runtime or via compose env_file
# ENV OPENAI_API_KEY=...

CMD ["bash"]
"#;
    t.to_string()
}

fn ai_claude_dockerfile_template() -> String {
    let t = r#"# syntax=docker/dockerfile:1.4
# Helper container for using Claude (Anthropic) workflows
FROM dockerfile:Dockerfile.dev

WORKDIR /workspace

ENV PIP_NO_CACHE_DIR=1

RUN apt-get update \
    && apt-get install -y --no-install-recommends python3 python3-pip git curl \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir anthropic

# Set your credentials at runtime or via compose env_file
# ENV ANTHROPIC_API_KEY=...

CMD ["bash"]
"#;
    t.to_string()
}

fn ai_goose_dockerfile_template() -> String {
    let t = r#"# syntax=docker/dockerfile:1.4
# Helper container for using Goose workflows
FROM dockerfile:Dockerfile.dev

WORKDIR /workspace

ENV PIP_NO_CACHE_DIR=1

RUN apt-get update \
    && apt-get install -y --no-install-recommends python3 python3-pip git curl \
    && rm -rf /var/lib/apt/lists/*

# There are multiple 'goose' projects; adjust as needed
# This installs the GooseAI Python SDK as a reasonable default
RUN pip install --no-cache-dir gooseai

# Provide credentials as env vars
# ENV GOOSEAI_API_KEY=...

CMD ["bash"]
"#;
    t.to_string()
}

fn ai_aider_dockerfile_template() -> String {
    let t = r#"# syntax=docker/dockerfile:1.4
# Helper container for using Aider (CLI AI pair programmer)
FROM dockerfile:Dockerfile.dev

WORKDIR /workspace

ENV PIP_NO_CACHE_DIR=1

RUN apt-get update \
    && apt-get install -y --no-install-recommends python3 python3-pip git curl \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir aider-chat

# Aider supports multiple backends; set the one you use
# ENV OPENAI_API_KEY=...
# ENV ANTHROPIC_API_KEY=...

CMD ["bash"]
"#;
    t.to_string()
}
