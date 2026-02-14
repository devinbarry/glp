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
        let scheme = if self.host.starts_with("127.0.0.1") || self.host.starts_with("localhost") {
            "http"
        } else {
            "https"
        };
        format!("{}://{}/api/v4{}", scheme, self.host, path)
    }

    pub fn project_encoded(&self) -> String {
        urlencoding::encode(&self.project).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::GlpError;
    use serial_test::serial;

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

    #[test]
    #[serial]
    fn resolve_token_from_gitlab_token_env() {
        unsafe {
            std::env::set_var("GITLAB_TOKEN", "tok-from-env");
            std::env::remove_var("GITLAB_PRIVATE_TOKEN");
        }
        let config = GlabConfig::default();
        let result = Config::resolve_token(&config, "gitlab.com");
        unsafe {
            std::env::remove_var("GITLAB_TOKEN");
        }
        assert_eq!(result.unwrap(), "tok-from-env");
    }

    #[test]
    #[serial]
    fn resolve_token_from_gitlab_private_token_env() {
        unsafe {
            std::env::remove_var("GITLAB_TOKEN");
            std::env::set_var("GITLAB_PRIVATE_TOKEN", "private-tok");
        }
        let config = GlabConfig::default();
        let result = Config::resolve_token(&config, "gitlab.com");
        unsafe {
            std::env::remove_var("GITLAB_PRIVATE_TOKEN");
        }
        assert_eq!(result.unwrap(), "private-tok");
    }

    #[test]
    #[serial]
    fn resolve_token_from_glab_config() {
        unsafe {
            std::env::remove_var("GITLAB_TOKEN");
            std::env::remove_var("GITLAB_PRIVATE_TOKEN");
        }
        let mut hosts = std::collections::HashMap::new();
        hosts.insert(
            "gitlab.com".to_string(),
            GlabHost {
                token: Some("glab-tok".to_string()),
            },
        );
        let config = GlabConfig {
            host: None,
            hosts: Some(hosts),
        };
        let result = Config::resolve_token(&config, "gitlab.com");
        assert_eq!(result.unwrap(), "glab-tok");
    }

    #[test]
    #[serial]
    fn resolve_token_missing_returns_no_token() {
        unsafe {
            std::env::remove_var("GITLAB_TOKEN");
            std::env::remove_var("GITLAB_PRIVATE_TOKEN");
        }
        let config = GlabConfig::default();
        let result = Config::resolve_token(&config, "gitlab.com");
        assert!(matches!(result, Err(GlpError::NoToken)));
    }

    #[test]
    #[serial]
    fn resolve_host_from_env_var() {
        unsafe {
            std::env::set_var("GITLAB_HOST", "env.gitlab.com");
        }
        let config = GlabConfig::default();
        let result = Config::resolve_host(&config);
        unsafe {
            std::env::remove_var("GITLAB_HOST");
        }
        assert_eq!(result, "env.gitlab.com");
    }

    #[test]
    #[serial]
    fn resolve_host_default_fallback() {
        unsafe {
            std::env::remove_var("GITLAB_HOST");
        }
        let config = GlabConfig::default();
        let result = Config::resolve_host(&config);
        // Result is either from git remote or the default "gitlab.com"
        // (depends on test environment). At minimum it should not be empty.
        assert!(!result.is_empty());
    }

    #[test]
    #[serial]
    fn resolve_host_env_takes_priority() {
        unsafe {
            std::env::set_var("GITLAB_HOST", "priority.gitlab.com");
        }
        let config = GlabConfig {
            host: Some("config.gitlab.com".to_string()),
            hosts: None,
        };
        let result = Config::resolve_host(&config);
        unsafe {
            std::env::remove_var("GITLAB_HOST");
        }
        assert_eq!(result, "priority.gitlab.com");
    }

    #[test]
    fn resolve_project_with_override() {
        let result = Config::resolve_project(Some("my-group/my-project".to_string()));
        assert_eq!(result.unwrap(), "my-group/my-project");
    }

    #[test]
    fn resolve_project_override_takes_priority() {
        // Even if git remote exists, override should win
        let result = Config::resolve_project(Some("override/project".to_string()));
        assert_eq!(result.unwrap(), "override/project");
    }

    #[test]
    fn api_url_localhost_uses_http() {
        let config = Config {
            token: "test".to_string(),
            host: "127.0.0.1:1234".to_string(),
            project: "group/project".to_string(),
        };
        assert_eq!(
            config.api_url("/projects/123/pipelines"),
            "http://127.0.0.1:1234/api/v4/projects/123/pipelines"
        );
    }
}
