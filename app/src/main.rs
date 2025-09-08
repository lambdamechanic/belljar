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
            let session = par_core::create_session(&args.label, &repo, args.branch, vec![])
                .map_err(|e| anyhow::anyhow!("create session failed: {e}"))?;
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
            println!("par checkout {} -- path={:?} label={:?}", target, path, label);
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
            println!("par open {} (placeholder)", label);
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
            println!("par send {} {:?} (placeholder)", target, command);
        }
        Commands::ControlCenter => {
            println!("par control-center (placeholder)");
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
