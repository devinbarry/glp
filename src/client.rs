use crate::config::Config;
use crate::error::{GlpError, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct GitLabClient {
    client: Client,
    config: Config,
}

impl GitLabClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.config.api_url(path);
        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GlpError::Api(format!("{}: {}", status, body)));
        }

        Ok(response.json().await?)
    }

    async fn get_text(&self, path: &str) -> Result<String> {
        let url = self.config.api_url(path);
        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GlpError::Api(format!("{}: {}", status, body)));
        }

        Ok(response.text().await?)
    }

    async fn post<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.config.api_url(path);
        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GlpError::Api(format!("{}: {}", status, body)));
        }

        Ok(response.json().await?)
    }

    pub async fn list_pipelines(&self, git_ref: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let project = self.config.project_encoded();
        let mut path = format!("/projects/{}/pipelines?per_page=1", project);
        if let Some(r) = git_ref {
            path.push_str(&format!("&ref={}", urlencoding::encode(r)));
        }
        self.get(&path).await
    }

    #[allow(dead_code)]
    pub async fn get_pipeline(&self, pipeline_id: u64) -> Result<serde_json::Value> {
        let project = self.config.project_encoded();
        let path = format!("/projects/{}/pipelines/{}", project, pipeline_id);
        self.get(&path).await
    }

    pub async fn get_pipeline_jobs(&self, pipeline_id: u64) -> Result<Vec<serde_json::Value>> {
        let project = self.config.project_encoded();
        let path = format!(
            "/projects/{}/pipelines/{}/jobs?per_page=100",
            project, pipeline_id
        );
        self.get(&path).await
    }

    pub async fn get_job_log(&self, job_id: u64) -> Result<String> {
        let project = self.config.project_encoded();
        let path = format!("/projects/{}/jobs/{}/trace", project, job_id);
        self.get_text(&path).await
    }

    pub async fn retry_job(&self, job_id: u64) -> Result<serde_json::Value> {
        let project = self.config.project_encoded();
        let path = format!("/projects/{}/jobs/{}/retry", project, job_id);
        self.post(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;

    fn test_client(server: &mockito::Server) -> GitLabClient {
        let config = Config {
            token: "test-token".to_string(),
            host: server.host_with_port(),
            project: "group/project".to_string(),
        };
        GitLabClient::new(config)
    }

    #[tokio::test]
    async fn list_pipelines_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v4/projects/group%2Fproject/pipelines")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "per_page".into(),
                "1".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"id":1,"ref":"main","status":"success"}]"#)
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.list_pipelines(None).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["id"], 1);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn list_pipelines_with_ref() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v4/projects/group%2Fproject/pipelines")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("per_page".into(), "1".into()),
                Matcher::UrlEncoded("ref".into(), "feature/branch".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"id":2,"ref":"feature/branch","status":"running"}]"#)
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.list_pipelines(Some("feature/branch")).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["ref"], "feature/branch");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn list_pipelines_api_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v4/projects/group%2Fproject/pipelines")
            .match_query(Matcher::Any)
            .with_status(403)
            .with_body("Forbidden")
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.list_pipelines(None).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, GlpError::Api(ref msg) if msg.contains("403") && msg.contains("Forbidden"))
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_pipeline_jobs_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v4/projects/group%2Fproject/pipelines/123/jobs")
            .match_query(Matcher::UrlEncoded("per_page".into(), "100".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"id":456,"name":"build","status":"success","stage":"build"}]"#)
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.get_pipeline_jobs(123).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "build");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_job_log_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v4/projects/group%2Fproject/jobs/456/trace")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("line1\nline2\nline3")
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.get_job_log(456).await.unwrap();

        assert_eq!(result, "line1\nline2\nline3");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn retry_job_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v4/projects/group%2Fproject/jobs/789/retry")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":790,"name":"test-job","status":"pending"}"#)
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.retry_job(789).await.unwrap();

        assert_eq!(result["id"], 790);
        assert_eq!(result["name"], "test-job");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn retry_job_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v4/projects/group%2Fproject/jobs/789/retry")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let client = test_client(&server);
        let result = client.retry_job(789).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GlpError::Api(ref msg) if msg.contains("500")));
        mock.assert_async().await;
    }
}
