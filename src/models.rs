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
