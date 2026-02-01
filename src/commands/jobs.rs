use crate::client::GitLabClient;
use crate::error::{GlpError, Result};
use crate::models::Job;
use crate::output;

pub async fn run(client: GitLabClient, pipeline_id: u64, json: bool) -> Result<()> {
    let job_values = client.get_pipeline_jobs(pipeline_id).await?;
    let jobs: Vec<Job> = job_values
        .into_iter()
        .filter_map(Job::from_json)
        .collect();

    if jobs.is_empty() {
        return Err(GlpError::Api(format!("No jobs found for pipeline {}", pipeline_id)));
    }

    if json {
        output::print_json(&jobs);
    } else {
        output::print_jobs_table(&jobs);
    }

    Ok(())
}
