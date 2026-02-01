use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlpError {
    #[error("GitLab API error: {0}")]
    Api(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("No GitLab token found. Set GITLAB_TOKEN or configure glab.")]
    NoToken,

    #[error("Could not determine project. Use --project or run from a git repo.")]
    NoProject,

    #[error("No pipeline found for ref '{0}'")]
    NoPipeline(String),

    #[error("Job {0} not found")]
    #[allow(dead_code)]
    JobNotFound(u64),
}

pub type Result<T> = std::result::Result<T, GlpError>;
