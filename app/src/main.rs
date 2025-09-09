use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "belljar", version, about = "belljar: session/worktree manager with per-session Docker Compose isolation")] 
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
    Send { target: String, command: Vec<String> },
    /// Show control center (placeholder)
    ControlCenter,
    /// Workspace subcommands (placeholder)
    Workspace { #[arg()] subcommand: Vec<String> },
    /// Print internal version details
    Version,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start(args) => {
            let repo = resolve_repo_path(args.path.as_deref())?;
            let mut session = par_core::create_session(&args.label, &repo, args.branch.clone(), vec![])
                .map_err(|e| anyhow::anyhow!("create session failed: {e}"))?;

            // Ensure worktree if repo is git
            if par_core::git::is_git_repo(&repo) {
                match par_core::git::ensure_worktree(&repo, &args.label, &args.branch) {
                    Ok(wt) => {
                        par_core::git::set_session_worktree(&mut session, wt).ok();
                    }
                    Err(e) => eprintln!("warning: worktree setup failed: {e}"),
                }
            }
            match par_core::compose::up(&session) {
                Ok(()) => {
                    println!(
                        "created session: {} (project: {}) [compose up]",
                        session.label, session.compose_project
                    );
                }
                Err(par_core::CoreError::NoComposeFiles) => {
                    println!(
                        "created session: {} (project: {}); no compose files found, skipping",
                        session.label, session.compose_project
                    );
                }
                Err(e) => {
                    eprintln!("warning: compose up failed: {e}");
                    println!("created session: {} (project: {})", session.label, session.compose_project);
                }
            }
        }
        Commands::Checkout { target, path, label } => {
            let repo = resolve_repo_path(path.as_deref())?;
            let label = label.unwrap_or_else(|| target.clone());
            let mut session = par_core::create_session(&label, &repo, Some(target.clone()), vec![])
                .map_err(|e| anyhow::anyhow!("create session failed: {e}"))?;
            if par_core::git::is_git_repo(&repo) {
                match par_core::git::ensure_worktree(&repo, &label, &Some(target.clone())) {
                    Ok(wt) => {
                        par_core::git::set_session_worktree(&mut session, wt).ok();
                    }
                    Err(e) => eprintln!("warning: worktree setup failed: {e}"),
                }
            }
            match par_core::compose::up(&session) {
                Ok(()) => println!(
                    "checked out: {} -> {} (project: {}) [compose up]",
                    label, target, session.compose_project
                ),
                Err(par_core::CoreError::NoComposeFiles) => println!(
                    "checked out: {} -> {} (project: {}); no compose files found, skipping",
                    label, target, session.compose_project
                ),
                Err(e) => eprintln!("warning: compose up failed: {e}"),
            }
        }
        Commands::Ls => {
            let reg = par_core::load_registry()
                .map_err(|e| anyhow::anyhow!("load registry failed: {e}"))?;
            if reg.sessions.is_empty() {
                println!("no sessions");
            } else {
                for s in reg.sessions {
                    println!("{}\t{}\t{}", s.label, s.repo_path.display(), s.compose_project);
                }
            }
        }
        Commands::Open { label } => {
            match par_core::find_session(&label) {
                Ok(Some(s)) => {
                    match par_core::tmux::ensure_session(&s) {
                        Ok(()) => {
                            // Try to attach. If tmux not found, provide guidance.
                            match par_core::tmux::attach(&s.tmux_session) {
                                Ok(()) => {}
                                Err(par_core::CoreError::TmuxNotFound) => {
                                    println!(
                                        "tmux not found; cd {} to work in this session",
                                        s.repo_path.display()
                                    );
                                }
                                Err(e) => eprintln!("failed to attach: {e}"),
                            }
                        }
                        Err(par_core::CoreError::TmuxNotFound) => {
                            println!(
                                "tmux not found; cd {} to work in this session",
                                s.repo_path.display()
                            );
                        }
                        Err(e) => eprintln!("failed to open session: {e}"),
                    }
                }
                Ok(None) => println!("no such session: {}", label),
                Err(e) => eprintln!("failed to load registry: {e}"),
            }
        }
        Commands::Rm { target } => {
            if target == "all" {
                let reg = par_core::load_registry()
                    .map_err(|e| anyhow::anyhow!("load registry failed: {e}"))?;
                for s in reg.sessions {
                    match par_core::compose::down(&s) {
                        Ok(()) | Err(par_core::CoreError::NoComposeFiles) => {}
                        Err(e) => eprintln!("warning: compose down failed for {}: {e}", s.label),
                    }
                    let _ = par_core::remove_session(&s.label);
                    println!("removed {}", s.label);
                }
            } else {
                if let Some(s) = par_core::find_session(&target)
                    .map_err(|e| anyhow::anyhow!("find session failed: {e}"))?
                {
                    match par_core::compose::down(&s) {
                        Ok(()) | Err(par_core::CoreError::NoComposeFiles) => {}
                        Err(e) => eprintln!("warning: compose down failed for {}: {e}", s.label),
                    }
                    par_core::remove_session(&target)
                        .map_err(|e| anyhow::anyhow!("remove failed: {e}"))?;
                    println!("removed {}", s.label);
                } else {
                    println!("no such session: {}", target);
                }
            }
        }
        Commands::Send { target, command } => {
            let cmd = command.join(" ");
            if target == "all" {
                match par_core::load_registry() {
                    Ok(reg) => {
                        for s in reg.sessions {
                            match par_core::tmux::ensure_session(&s) {
                                Ok(()) => {
                                    if let Err(e) = par_core::tmux::send_keys(&s.tmux_session, &cmd) {
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
                match par_core::find_session(&target) {
                    Ok(Some(s)) => {
                        match par_core::tmux::ensure_session(&s) {
                            Ok(()) => {
                                if let Err(e) = par_core::tmux::send_keys(&s.tmux_session, &cmd) {
                                    eprintln!("send failed: {e}");
                                }
                            }
                            Err(e) => eprintln!("ensure session failed: {e}"),
                        }
                    }
                    Ok(None) => println!("no such session: {}", target),
                    Err(e) => eprintln!("failed to load registry: {e}"),
                }
            }
        }
        Commands::ControlCenter => {
            // Create a tmux session named "belljar-cc" with one window per session
            let cc_name = "belljar-cc";
            match par_core::load_registry() {
                Ok(reg) => {
                    if reg.sessions.is_empty() {
                        println!("no sessions to show");
                    } else {
                        // Use the first session's repo as the base cwd
                        let base = &reg.sessions[0].repo_path;
                        match par_core::tmux::ensure_named_session(cc_name, base) {
                            Ok(()) => {
                                for s in reg.sessions {
                                    // Try to create a window per session label
                                    if let Err(e) = par_core::tmux::new_window(cc_name, &s.label, &s.repo_path) {
                                        eprintln!("failed to create window for {}: {e}", s.label);
                                    }
                                }
                                // Optional: choose a tiled layout
                                let _ = par_core::tmux::select_layout(cc_name, "tiled");
                                // Attach to control center
                                if let Err(e) = par_core::tmux::attach(cc_name) {
                                    eprintln!("failed to attach control center: {e}");
                                }
                            }
                            Err(par_core::CoreError::TmuxNotFound) => {
                                println!("tmux not found; control-center requires tmux");
                            }
                            Err(e) => eprintln!("failed to init control center: {e}"),
                        }
                    }
                }
                Err(e) => eprintln!("failed to load registry: {e}"),
            }
        }
        Commands::Workspace { subcommand } => {
            println!("par workspace {:?} (placeholder)", subcommand);
        }
        Commands::Version => {
            println!("par {} (core {})", env!("CARGO_PKG_VERSION"), par_core::version());
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
