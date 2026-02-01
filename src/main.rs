mod client;
mod commands;
mod config;
mod error;
mod models;
mod output;

use clap::{Parser, Subcommand};
use client::GitLabClient;
use config::Config;
use error::Result;

#[derive(Parser)]
#[command(name = "glp", version, about = "GitLab Pipeline CLI")]
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
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status {
            json,
            project,
            git_ref,
        } => {
            let config = Config::load(project)?;
            let client = GitLabClient::new(config);
            commands::status::run(client, git_ref, json).await
        }
        Commands::Jobs {
            pipeline_id,
            json,
            project,
        } => {
            let config = Config::load(project)?;
            let client = GitLabClient::new(config);
            commands::jobs::run(client, pipeline_id, json).await
        }
        Commands::Log {
            job_id,
            tail,
            project,
        } => {
            let config = Config::load(project)?;
            let client = GitLabClient::new(config);
            commands::log::run(client, job_id, tail).await
        }
        Commands::Retry { job_id, project } => {
            let config = Config::load(project)?;
            let client = GitLabClient::new(config);
            commands::retry::run(client, job_id).await
        }
    }
}
