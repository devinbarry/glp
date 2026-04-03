use crate::client::GitLabClient;
use crate::error::{GlpError, Result};
use crate::models::{Job, Pipeline};
use crate::output;

pub async fn run(client: GitLabClient, git_ref: Option<String>, json: bool) -> Result<()> {
    // Get current branch if no ref specified
    let resolved = match git_ref {
        Some(r) => ResolvedRef::Branch(r),
        None => resolve_head()?,
    };

    // Get latest pipeline for this ref (or SHA if detached HEAD)
    let (pipelines, ref_label) = match &resolved {
        ResolvedRef::Branch(branch) => {
            let pipelines = client.list_pipelines(Some(branch), None).await?;
            (pipelines, branch.clone())
        }
        ResolvedRef::DetachedSha(sha) => {
            let pipelines = client.list_pipelines(None, Some(sha)).await?;
            (pipelines, sha.clone())
        }
    };
    let pipeline = find_pipeline(pipelines, &ref_label)?;

    // Get jobs for this pipeline
    let job_values = client.get_pipeline_jobs(pipeline.id).await?;
    let jobs: Vec<Job> = job_values.into_iter().filter_map(Job::from_json).collect();

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

enum ResolvedRef {
    Branch(String),
    DetachedSha(String),
}

fn find_pipeline(pipelines: Vec<serde_json::Value>, ref_label: &str) -> Result<Pipeline> {
    let value = pipelines
        .into_iter()
        .next()
        .ok_or_else(|| GlpError::NoPipeline(ref_label.to_string()))?;
    Pipeline::from_json(value).ok_or_else(|| GlpError::Api("Invalid pipeline response".to_string()))
}

fn resolve_head() -> Result<ResolvedRef> {
    let branch_output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GlpError::Config("Failed to get current branch".to_string()))?;

    if !branch_output.status.success() {
        return Err(GlpError::Config("Not in a git repository".to_string()));
    }

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    if branch != "HEAD" {
        return Ok(ResolvedRef::Branch(branch));
    }

    // Detached HEAD — resolve the SHA so we can query by commit
    let sha_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|_| GlpError::Config("Failed to get HEAD SHA".to_string()))?;

    if !sha_output.status.success() {
        return Err(GlpError::Config("Failed to get HEAD SHA".to_string()));
    }

    let sha = String::from_utf8_lossy(&sha_output.stdout)
        .trim()
        .to_string();
    Ok(ResolvedRef::DetachedSha(sha))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_pipeline_json() -> serde_json::Value {
        json!({
            "id": 741,
            "ref": "main",
            "status": "success",
            "duration": 120.0,
            "created_at": "2026-01-31T12:00:00Z",
            "web_url": "https://gitlab.com/group/project/-/pipelines/741"
        })
    }

    #[test]
    fn find_pipeline_returns_first() {
        let pipelines = vec![valid_pipeline_json()];
        let result = find_pipeline(pipelines, "main").unwrap();
        assert_eq!(result.id, 741);
        assert_eq!(result.git_ref, "main");
    }

    #[test]
    fn find_pipeline_empty_returns_no_pipeline_error() {
        let result = find_pipeline(vec![], "main");
        assert!(matches!(result, Err(GlpError::NoPipeline(ref r)) if r == "main"));
    }

    #[test]
    fn find_pipeline_invalid_json_returns_api_error() {
        let pipelines = vec![json!({"bad": true})];
        let result = find_pipeline(pipelines, "main");
        assert!(matches!(result, Err(GlpError::Api(_))));
    }

    #[test]
    fn find_pipeline_empty_with_sha_includes_sha_in_error() {
        let sha = "abc123def456789";
        let result = find_pipeline(vec![], sha);
        assert!(matches!(result, Err(GlpError::NoPipeline(ref r)) if r == sha));
    }
}
