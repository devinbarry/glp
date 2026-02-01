use crate::error::{GlpError, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub host: String,
    pub project: String,
}

#[derive(Deserialize, Debug)]
struct GlabConfig {
    hosts: Option<std::collections::HashMap<String, GlabHost>>,
}

#[derive(Deserialize, Debug)]
struct GlabHost {
    token: Option<String>,
}

impl Config {
    pub fn load(project_override: Option<String>) -> Result<Self> {
        let token = Self::resolve_token()?;
        let host = Self::resolve_host();
        let project = Self::resolve_project(project_override)?;

        Ok(Config {
            token,
            host,
            project,
        })
    }

    fn resolve_token() -> Result<String> {
        // 1. Environment variables
        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            return Ok(token);
        }
        if let Ok(token) = std::env::var("GITLAB_PRIVATE_TOKEN") {
            return Ok(token);
        }

        // 2. glab config
        if let Some(token) = Self::read_glab_token() {
            return Ok(token);
        }

        Err(GlpError::NoToken)
    }

    fn resolve_host() -> String {
        // 1. Environment variable
        if let Ok(host) = std::env::var("GITLAB_HOST") {
            return host;
        }

        // 2. glab config (simplified - just use default)
        // 3. Default
        "gitlab.com".to_string()
    }

    fn resolve_project(override_project: Option<String>) -> Result<String> {
        // 1. Override flag
        if let Some(project) = override_project {
            return Ok(project);
        }

        // 2. Git remote origin
        if let Some(project) = Self::parse_git_remote() {
            return Ok(project);
        }

        Err(GlpError::NoProject)
    }

    fn glab_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("glab-cli").join("config.yml"))
    }

    fn read_glab_token() -> Option<String> {
        let path = Self::glab_config_path()?;
        let content = std::fs::read_to_string(path).ok()?;
        let config: GlabConfig = yaml_serde::from_str(&content).ok()?;

        // Get token from first host (simplified)
        let hosts = config.hosts?;
        for (_host, host_config) in hosts {
            if let Some(token) = host_config.token {
                return Some(token);
            }
        }
        None
    }

    fn parse_git_remote() -> Option<String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Self::extract_project_from_url(&url)
    }

    fn extract_project_from_url(url: &str) -> Option<String> {
        // SSH: git@gitlab.com:group/project.git
        if url.starts_with("git@") {
            let path = url.split(':').nth(1)?;
            return Some(path.trim_end_matches(".git").to_string());
        }

        // HTTPS: https://gitlab.com/group/project.git
        if let Ok(parsed) = url::Url::parse(url) {
            let path = parsed
                .path()
                .trim_start_matches('/')
                .trim_end_matches(".git");
            return Some(path.to_string());
        }

        None
    }

    pub fn api_url(&self, path: &str) -> String {
        format!("https://{}/api/v4{}", self.host, path)
    }

    pub fn project_encoded(&self) -> String {
        urlencoding::encode(&self.project).to_string()
    }
}
