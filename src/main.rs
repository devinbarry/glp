mod client;
mod config;
mod error;
mod models;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "glp")]
#[command(about = "GitLab Pipeline CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show pipeline status for current branch
    Status {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        project: Option<String>,
        #[arg(long, name = "ref")]
        git_ref: Option<String>,
    },
    /// List jobs in a pipeline
    Jobs {
        pipeline_id: u64,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        project: Option<String>,
    },
    /// Fetch job log
    Log {
        job_id: u64,
        #[arg(long)]
        tail: Option<usize>,
        #[arg(long)]
        project: Option<String>,
    },
    /// Retry a failed job
    Retry {
        job_id: u64,
        #[arg(long)]
        project: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { .. } => println!("status not implemented"),
        Commands::Jobs { .. } => println!("jobs not implemented"),
        Commands::Log { .. } => println!("log not implemented"),
        Commands::Retry { .. } => println!("retry not implemented"),
    }
}
