use crate::client::GitLabClient;
use crate::error::Result;

pub async fn run(client: GitLabClient, job_id: u64, tail: Option<usize>) -> Result<()> {
    let log = client.get_job_log(job_id).await?;

    match tail {
        Some(n) => {
            let lines: Vec<&str> = log.lines().collect();
            let start = lines.len().saturating_sub(n);
            for line in &lines[start..] {
                println!("{}", line);
            }
        }
        None => {
            print!("{}", log);
        }
    }

    Ok(())
}
