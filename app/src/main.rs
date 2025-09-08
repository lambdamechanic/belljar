use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "par", version, about = "par-rs: parallel task runner with per-session Docker Compose isolation")] 
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run tasks defined in the config file
    Run {
        /// Path to config file (YAML/TOML)
        #[arg(short, long)]
        config: Option<PathBuf>,
        /// Comma-separated services to include (e.g., postgres,redis)
        #[arg(long, value_delimiter = ',')]
        with: Vec<String>,
        /// Keep the docker-compose stack running after completion
        #[arg(long)]
        keep: bool,
        /// Max parallelism for tasks
        #[arg(short, long)]
        parallel: Option<usize>,
        /// Retry failed tasks up to N times
        #[arg(long)]
        retry: Option<usize>,
    },
    /// Print internal version details
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Run { config: None, with: vec![], keep: false, parallel: None, retry: None }) {
        Commands::Run { config, with, keep, parallel, retry } => {
            // Placeholder: wire into par_core when implemented
            println!(
                "par run -- config={:?} with={:?} keep={} parallel={:?} retry={:?}",
                config, with, keep, parallel, retry
            );
        }
        Commands::Version => {
            println!("par {} (core {})", env!("CARGO_PKG_VERSION"), par_core::version());
        }
    }
    Ok(())
}

