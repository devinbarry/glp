use crate::client::GitLabClient;
use crate::error::{GlpError, Result};

pub async fn run(client: GitLabClient, job_id: u64) -> Result<()> {
    let response = client.retry_job(job_id).await?;

    let new_id = response
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| GlpError::Api("Invalid retry response".to_string()))?;

    let name = response
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("Retried job {} ({}) - new job ID: {}", job_id, name, new_id);

    Ok(())
}
