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

#[derive(Deserialize, Debug, Default)]
struct GlabConfig {
    host: Option<String>,
    hosts: Option<std::collections::HashMap<String, GlabHost>>,
}

#[derive(Deserialize, Debug, Default)]
struct GlabHost {
    token: Option<String>,
}

impl Config {
    pub fn load(project_override: Option<String>) -> Result<Self> {
        let glab_config = Self::read_glab_config().unwrap_or_default();
        let host = Self::resolve_host(&glab_config);
        let token = Self::resolve_token(&glab_config, &host)?;
        let project = Self::resolve_project(project_override)?;

        Ok(Config {
            token,
            host,
            project,
        })
    }

    fn resolve_token(glab_config: &GlabConfig, host: &str) -> Result<String> {
        // 1. Environment variables
        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            return Ok(token);
        }
        if let Ok(token) = std::env::var("GITLAB_PRIVATE_TOKEN") {
            return Ok(token);
        }

        // 2. glab config - get token for specific host
        if let Some(hosts) = &glab_config.hosts {
            if let Some(host_config) = hosts.get(host) {
                if let Some(token) = &host_config.token {
                    if !token.is_empty() {
                        return Ok(token.clone());
                    }
                }
            }
        }

        Err(GlpError::NoToken)
    }

    fn resolve_host(glab_config: &GlabConfig) -> String {
        // 1. Environment variable
        if let Ok(host) = std::env::var("GITLAB_HOST") {
            return host;
        }

        // 2. Host from git remote
        if let Some(host) = Self::extract_host_from_remote() {
            return host;
        }

        // 3. glab config default host
        if let Some(host) = &glab_config.host {
            return host.clone();
        }

        // 4. Default
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

    fn glab_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // macOS: ~/Library/Application Support/glab-cli/config.yml
        if let Some(data_dir) = dirs::data_dir() {
            paths.push(data_dir.join("glab-cli").join("config.yml"));
        }

        // Linux/others: ~/.config/glab-cli/config.yml
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("glab-cli").join("config.yml"));
        }

        paths
    }

    fn read_glab_config() -> Option<GlabConfig> {
        for path in Self::glab_config_paths() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = yaml_serde::from_str(&content) {
                    return Some(config);
                }
            }
        }
        None
    }

    fn extract_host_from_remote() -> Option<String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Self::extract_host_from_url(&url)
    }

    fn extract_host_from_url(url: &str) -> Option<String> {
        // SSH: git@gitlab.com:group/project.git
        if url.starts_with("git@") {
            let host = url.strip_prefix("git@")?.split(':').next()?;
            return Some(host.to_string());
        }

        // HTTPS: https://gitlab.com/group/project.git
        if let Ok(parsed) = url::Url::parse(url) {
            return parsed.host_str().map(|h| h.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_project_from_ssh_url() {
        let url = "git@gitlab.com:group/project.git";
        assert_eq!(
            Config::extract_project_from_url(url),
            Some("group/project".to_string())
        );
    }

    #[test]
    fn extract_project_from_ssh_url_nested() {
        let url = "git@gitlab.com:group/subgroup/project.git";
        assert_eq!(
            Config::extract_project_from_url(url),
            Some("group/subgroup/project".to_string())
        );
    }

    #[test]
    fn extract_project_from_https_url() {
        let url = "https://gitlab.com/group/project.git";
        assert_eq!(
            Config::extract_project_from_url(url),
            Some("group/project".to_string())
        );
    }

    #[test]
    fn extract_project_from_https_url_no_git_suffix() {
        let url = "https://gitlab.com/group/project";
        assert_eq!(
            Config::extract_project_from_url(url),
            Some("group/project".to_string())
        );
    }

    #[test]
    fn extract_host_from_ssh_url() {
        let url = "git@gitlab.example.com:group/project.git";
        assert_eq!(
            Config::extract_host_from_url(url),
            Some("gitlab.example.com".to_string())
        );
    }

    #[test]
    fn extract_host_from_https_url() {
        let url = "https://gitlab.example.com/group/project.git";
        assert_eq!(
            Config::extract_host_from_url(url),
            Some("gitlab.example.com".to_string())
        );
    }

    #[test]
    fn api_url_builds_correctly() {
        let config = Config {
            token: "test".to_string(),
            host: "gitlab.example.com".to_string(),
            project: "group/project".to_string(),
        };
        assert_eq!(
            config.api_url("/projects/123/pipelines"),
            "https://gitlab.example.com/api/v4/projects/123/pipelines"
        );
    }

    #[test]
    fn project_encoded_encodes_slashes() {
        let config = Config {
            token: "test".to_string(),
            host: "gitlab.com".to_string(),
            project: "group/subgroup/project".to_string(),
        };
        assert_eq!(config.project_encoded(), "group%2Fsubgroup%2Fproject");
    }
}
