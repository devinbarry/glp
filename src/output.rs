use crate::models::{Job, Pipeline};
use colored::Colorize;

pub fn status_color(status: &str) -> colored::ColoredString {
    match status {
        "success" | "passed" => status.green(),
        "failed" => status.red(),
        "running" => status.blue(),
        "pending" => status.yellow(),
        "canceled" | "cancelled" => status.magenta(),
        "skipped" => status.dimmed(),
        _ => status.normal(),
    }
}

pub fn print_pipeline_header(pipeline: &Pipeline) {
    let status = status_color(&pipeline.status);
    println!(
        "Pipeline #{} ({}) - {} [{}]",
        pipeline.id,
        pipeline.git_ref,
        status,
        pipeline.duration_str()
    );
    println!();
}

pub fn print_jobs_table(jobs: &[Job]) {
    println!(
        "{:<20} {:<8} {:<10} {:<10} STAGE",
        "JOB", "ID", "STATUS", "DURATION"
    );

    for job in jobs {
        let status = status_color(&job.status);
        println!(
            "{:<20} {:<8} {:<10} {:<10} {}",
            truncate(&job.name, 20),
            job.id,
            status,
            job.duration_str(),
            job.stage
        );
    }
}

pub fn print_status_table(jobs: &[Job]) {
    println!("{:<12} {:<20} {:<10} DURATION", "STAGE", "JOB", "STATUS");

    for job in jobs {
        let status = status_color(&job.status);
        println!(
            "{:<12} {:<20} {:<10} {}",
            job.stage,
            truncate(&job.name, 20),
            status,
            job.duration_str()
        );
    }
}

pub fn print_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string_adds_ellipsis() {
        assert_eq!(truncate("hello world!", 8), "hello...");
    }

    #[test]
    fn truncate_min_length() {
        assert_eq!(truncate("abcdef", 3), "...");
        assert_eq!(truncate("abcdef", 4), "a...");
    }

    #[test]
    fn status_color_success_variants() {
        assert_eq!(status_color("success").deref(), "success");
        assert_eq!(status_color("passed").deref(), "passed");
    }

    #[test]
    fn status_color_all_known_statuses() {
        for s in &[
            "failed",
            "running",
            "pending",
            "canceled",
            "cancelled",
            "skipped",
        ] {
            assert_eq!(status_color(s).deref(), *s);
        }
    }

    #[test]
    fn status_color_unknown_falls_through() {
        assert_eq!(status_color("manual").deref(), "manual");
    }
}
