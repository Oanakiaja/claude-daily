use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::load_config;
use crate::jobs::{JobManager, JobStatus};

/// List all jobs
pub async fn list(all: bool) -> Result<()> {
    let config = load_config()?;
    let manager = JobManager::new(&config)?;

    let jobs = manager.list(all)?;

    if jobs.is_empty() {
        if all {
            println!("No jobs found.");
        } else {
            println!("No running jobs. Use --all to see completed jobs.");
        }
        return Ok(());
    }

    // Print header
    println!(
        "{:<28} {:<12} {:<20} {:<10}",
        "ID".bold(),
        "STATUS".bold(),
        "TASK".bold(),
        "ELAPSED".bold()
    );
    println!("{}", "-".repeat(72));

    for job in jobs {
        let status_str = match &job.status {
            JobStatus::Running => "Running".green().to_string(),
            JobStatus::Completed => "Completed".blue().to_string(),
            JobStatus::Failed { .. } => "Failed".red().to_string(),
        };

        let task_display = if job.task_name.len() > 18 {
            format!("{}...", &job.task_name[..15])
        } else {
            job.task_name.clone()
        };

        println!(
            "{:<28} {:<12} {:<20} {:<10}",
            job.id,
            status_str,
            task_display,
            job.elapsed_human()
        );
    }

    Ok(())
}

/// Show log for a job
pub async fn log(job_id: String, tail: Option<usize>, follow: bool) -> Result<()> {
    let config = load_config()?;
    let manager = JobManager::new(&config)?;

    // Verify job exists
    let job = manager.load_job(&job_id).context("Job not found")?;

    println!(
        "{} {} ({})",
        "Job:".bold(),
        job.id,
        job.status.to_string().cyan()
    );
    println!("{} {}", "Task:".bold(), job.task_name);
    println!(
        "{} {}",
        "Started:".bold(),
        job.started_at.format("%Y-%m-%d %H:%M:%S")
    );
    if let Some(finished) = job.finished_at {
        println!(
            "{} {}",
            "Finished:".bold(),
            finished.format("%Y-%m-%d %H:%M:%S")
        );
    }
    println!("{}", "-".repeat(50));

    if follow && job.status == JobStatus::Running {
        // Follow mode - continuously read log
        follow_log(&manager, &job_id).await?;
    } else {
        // One-shot read
        match manager.read_log(&job_id, tail) {
            Ok(content) => {
                if content.is_empty() {
                    println!("{}", "(no log output)".dimmed());
                } else {
                    println!("{}", content);
                }
            }
            Err(_) => {
                println!("{}", "(log file not found)".dimmed());
            }
        }
    }

    Ok(())
}

/// Follow log output in real-time
async fn follow_log(manager: &JobManager, job_id: &str) -> Result<()> {
    use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
    use std::time::Duration;

    let log_path = manager.log_path(job_id);

    // Wait for log file to exist
    let mut attempts = 0;
    while !log_path.exists() && attempts < 10 {
        tokio::time::sleep(Duration::from_millis(200)).await;
        attempts += 1;
    }

    if !log_path.exists() {
        println!("{}", "(waiting for log output...)".dimmed());
    }

    let mut last_pos = 0u64;

    loop {
        // Check if job is still running
        if let Ok(job) = manager.load_job(job_id) {
            if job.status != JobStatus::Running {
                // Print remaining content and exit
                if let Ok(file) = std::fs::File::open(&log_path) {
                    let mut reader = BufReader::new(file);
                    reader.seek(SeekFrom::Start(last_pos))?;
                    for line in reader.lines().map_while(Result::ok) {
                        println!("{}", line);
                    }
                }
                println!("\n{} {}", "Job finished:".bold(), job.status);
                break;
            }
        }

        // Read new content
        if let Ok(file) = std::fs::File::open(&log_path) {
            let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);

            if file_len > last_pos {
                let mut reader = BufReader::new(file);
                reader.seek(SeekFrom::Start(last_pos))?;

                let mut buf = String::new();
                if reader.read_to_string(&mut buf).is_ok() && !buf.is_empty() {
                    print!("{}", buf);
                }

                last_pos = file_len;
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

/// Kill a running job
pub async fn kill(job_id: String) -> Result<()> {
    let config = load_config()?;
    let manager = JobManager::new(&config)?;

    let job = manager.load_job(&job_id).context("Job not found")?;

    if job.status != JobStatus::Running {
        println!(
            "{} Job {} is not running (status: {})",
            "Warning:".yellow(),
            job_id,
            job.status
        );
        return Ok(());
    }

    if manager.kill(&job_id)? {
        println!(
            "{} Killed job {} (PID: {})",
            "Success:".green(),
            job_id,
            job.pid
        );
    } else {
        println!(
            "{} Failed to kill job {} (PID: {}). Process may have already exited.",
            "Error:".red(),
            job_id,
            job.pid
        );
    }

    Ok(())
}

/// Cleanup old jobs
pub async fn cleanup(days: u32, dry_run: bool) -> Result<()> {
    let config = load_config()?;
    let manager = JobManager::new(&config)?;

    if dry_run {
        let jobs = manager.list(true)?;
        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let to_remove: Vec<_> = jobs
            .iter()
            .filter(|j| j.status != JobStatus::Running && j.started_at < cutoff)
            .collect();

        if to_remove.is_empty() {
            println!("No jobs to clean up.");
        } else {
            println!("Would remove {} jobs:", to_remove.len());
            for job in to_remove {
                println!("  - {} ({})", job.id, job.task_name);
            }
        }
    } else {
        let removed = manager.cleanup(days)?;
        println!(
            "{} Removed {} old job(s) (older than {} days)",
            "Success:".green(),
            removed,
            days
        );
    }

    Ok(())
}
