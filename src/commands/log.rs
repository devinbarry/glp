use crate::client::GitLabClient;
use crate::error::Result;

pub async fn run(client: GitLabClient, job_id: u64, tail: Option<usize>) -> Result<()> {
    let log = client.get_job_log(job_id).await?;

    match tail {
        Some(n) => {
            for line in tail_lines(&log, n) {
                println!("{}", line);
            }
        }
        None => {
            print!("{}", log);
        }
    }

    Ok(())
}

fn tail_lines(log: &str, n: usize) -> Vec<&str> {
    let lines: Vec<&str> = log.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_lines_returns_last_n() {
        assert_eq!(tail_lines("a\nb\nc\nd\ne", 3), vec!["c", "d", "e"]);
    }

    #[test]
    fn tail_lines_n_exceeds_total() {
        assert_eq!(tail_lines("a\nb", 10), vec!["a", "b"]);
    }

    #[test]
    fn tail_lines_zero() {
        let result = tail_lines("a\nb\nc", 0);
        assert!(result.is_empty());
    }

    #[test]
    fn tail_lines_empty_input() {
        let result = tail_lines("", 5);
        assert!(result.is_empty());
    }

    #[test]
    fn tail_lines_single_line() {
        assert_eq!(tail_lines("single", 1), vec!["single"]);
    }
}
