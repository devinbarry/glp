use crate::client::GitLabClient;
use crate::error::{GlpError, Result};
use crate::models::{Job, Pipeline};
use crate::output;

pub async fn run(client: GitLabClient, git_ref: Option<String>, json: bool) -> Result<()> {
    // Get current branch if no ref specified
    let git_ref = match git_ref {
        Some(r) => r,
        None => get_current_branch()?,
    };

    // Get latest pipeline for this ref
    let pipelines = client.list_pipelines(Some(&git_ref)).await?;
    let pipeline_value = pipelines.into_iter().next()
        .ok_or_else(|| GlpError::NoPipeline(git_ref.clone()))?;

    let pipeline = Pipeline::from_json(pipeline_value.clone())
        .ok_or_else(|| GlpError::Api("Invalid pipeline response".to_string()))?;

    // Get jobs for this pipeline
    let job_values = client.get_pipeline_jobs(pipeline.id).await?;
    let jobs: Vec<Job> = job_values
        .into_iter()
        .filter_map(Job::from_json)
        .collect();

    if json {
        let output = serde_json::json!({
            "pipeline": pipeline,
            "jobs": jobs
        });
        output::print_json(&output);
    } else {
        output::print_pipeline_header(&pipeline);
        output::print_status_table(&jobs);
    }

    Ok(())
}

fn get_current_branch() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GlpError::Config("Failed to get current branch".to_string()))?;

    if !output.status.success() {
        return Err(GlpError::Config("Not in a git repository".to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
