use crate::client::GitLabClient;
use crate::error::{GlpError, Result};

pub async fn run(client: GitLabClient, job_id: u64) -> Result<()> {
    let response = client.retry_job(job_id).await?;
    let (new_id, name) = parse_retry_response(&response)?;

    println!("Retried job {} ({}) - new job ID: {}", job_id, name, new_id);

    Ok(())
}

fn parse_retry_response(response: &serde_json::Value) -> Result<(u64, String)> {
    let new_id = response
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| GlpError::Api("Invalid retry response".to_string()))?;

    let name = response
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    Ok((new_id, name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_retry_response_valid() {
        let resp = json!({"id": 999, "name": "test-job"});
        let (id, name) = parse_retry_response(&resp).unwrap();
        assert_eq!(id, 999);
        assert_eq!(name, "test-job");
    }

    #[test]
    fn parse_retry_response_missing_id() {
        let resp = json!({"name": "test-job"});
        assert!(parse_retry_response(&resp).is_err());
    }

    #[test]
    fn parse_retry_response_missing_name_defaults() {
        let resp = json!({"id": 100});
        let (id, name) = parse_retry_response(&resp).unwrap();
        assert_eq!(id, 100);
        assert_eq!(name, "unknown");
    }

    #[test]
    fn parse_retry_response_non_numeric_id() {
        let resp = json!({"id": "abc", "name": "x"});
        assert!(parse_retry_response(&resp).is_err());
    }
}
