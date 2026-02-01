use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline {
    pub id: u64,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub status: String,
    pub duration: Option<f64>,
    pub created_at: String,
    pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Job {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub stage: String,
    pub duration: Option<f64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub web_url: String,
}

impl Pipeline {
    pub fn from_json(value: serde_json::Value) -> Option<Self> {
        serde_json::from_value(value).ok()
    }

    pub fn duration_str(&self) -> String {
        match self.duration {
            Some(d) => format_duration(d),
            None => "-".to_string(),
        }
    }
}

impl Job {
    pub fn from_json(value: serde_json::Value) -> Option<Self> {
        serde_json::from_value(value).ok()
    }

    pub fn duration_str(&self) -> String {
        match self.duration {
            Some(d) => format_duration(d),
            None => "-".to_string(),
        }
    }
}

fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}m {:02}s", mins, secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(154.0), "2m 34s");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0.0), "0m 00s");
    }

    #[test]
    fn format_duration_seconds_only() {
        assert_eq!(format_duration(45.0), "0m 45s");
    }

    #[test]
    fn format_duration_large() {
        assert_eq!(format_duration(3661.0), "61m 01s");
    }

    #[test]
    fn pipeline_from_json_valid() {
        let json = json!({
            "id": 741,
            "ref": "master",
            "status": "failed",
            "duration": 154.5,
            "created_at": "2026-01-31T12:00:00Z",
            "web_url": "https://gitlab.com/group/project/-/pipelines/741"
        });
        let pipeline = Pipeline::from_json(json).unwrap();
        assert_eq!(pipeline.id, 741);
        assert_eq!(pipeline.git_ref, "master");
        assert_eq!(pipeline.status, "failed");
        assert_eq!(pipeline.duration, Some(154.5));
    }

    #[test]
    fn pipeline_from_json_null_duration() {
        let json = json!({
            "id": 742,
            "ref": "feature",
            "status": "running",
            "duration": null,
            "created_at": "2026-01-31T12:00:00Z",
            "web_url": "https://gitlab.com/group/project/-/pipelines/742"
        });
        let pipeline = Pipeline::from_json(json).unwrap();
        assert_eq!(pipeline.duration, None);
        assert_eq!(pipeline.duration_str(), "-");
    }

    #[test]
    fn job_from_json_valid() {
        let json = json!({
            "id": 2659,
            "name": "test_longline",
            "status": "failed",
            "stage": "test",
            "duration": 45.0,
            "created_at": "2026-01-31T12:00:00Z",
            "started_at": "2026-01-31T12:01:00Z",
            "finished_at": "2026-01-31T12:01:45Z",
            "web_url": "https://gitlab.com/group/project/-/jobs/2659"
        });
        let job = Job::from_json(json).unwrap();
        assert_eq!(job.id, 2659);
        assert_eq!(job.name, "test_longline");
        assert_eq!(job.stage, "test");
        assert_eq!(job.duration_str(), "0m 45s");
    }

    #[test]
    fn pipeline_from_json_invalid_returns_none() {
        let json = json!({"invalid": "data"});
        assert!(Pipeline::from_json(json).is_none());
    }
}
