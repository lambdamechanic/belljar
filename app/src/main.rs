use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "par", version, about = "par-rs: session/worktree manager with per-session Docker Compose isolation")] 
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
    /// Comma-separated services to include (e.g., postgres,redis)
    #[arg(long, value_delimiter = ',')]
    with: Vec<String>,
    /// Keep the docker-compose stack running after completion
    #[arg(long)]
    keep: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start(args) => {
            println!(
                "par start {} -- path={:?} branch={:?} with={:?} keep={}",
                args.label, args.path, args.branch, args.with, args.keep
            );
        }
        Commands::Checkout { target, path, label } => {
            println!("par checkout {} -- path={:?} label={:?}", target, path, label);
        }
        Commands::Ls => {
            println!("par ls (placeholder)");
        }
        Commands::Open { label } => {
            println!("par open {} (placeholder)", label);
        }
        Commands::Rm { target } => {
            println!("par rm {} (placeholder)", target);
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
