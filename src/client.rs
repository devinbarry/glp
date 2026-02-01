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
        let response = self.client
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
        let response = self.client
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
        let response = self.client
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
        let path = format!("/projects/{}/pipelines/{}/jobs?per_page=100", project, pipeline_id);
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
