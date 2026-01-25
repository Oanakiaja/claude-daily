use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

/// Maximum log file size in bytes (1MB)
const MAX_LOG_SIZE: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Running,
    Completed,
    Failed { error: String },
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed { error } => write!(f, "Failed: {}", error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum JobType {
    SessionEnd,
    AutoSummarize,
    #[default]
    Manual,
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::SessionEnd => write!(f, "Session End"),
            JobType::AutoSummarize => write!(f, "Auto Summarize"),
            JobType::Manual => write!(f, "Manual"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub pid: u32,
    pub task_name: String,
    pub transcript_path: PathBuf,
    pub started_at: DateTime<Local>,
    pub finished_at: Option<DateTime<Local>>,
    pub status: JobStatus,
    #[serde(default)]
    pub job_type: JobType,
}

impl JobInfo {
    /// Check if the job process is still alive
    pub fn is_alive(&self) -> bool {
        is_process_alive(self.pid)
    }

    /// Get duration since job started
    pub fn elapsed(&self) -> chrono::Duration {
        let end = self.finished_at.unwrap_or_else(Local::now);
        end - self.started_at
    }

    /// Format elapsed time as human-readable string
    pub fn elapsed_human(&self) -> String {
        let elapsed = self.elapsed();
        let secs = elapsed.num_seconds();

        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }
}

pub struct JobManager {
    jobs_dir: PathBuf,
}

impl JobManager {
    /// Create a new JobManager
    pub fn new(config: &Config) -> Result<Self> {
        let jobs_dir = config.storage_path().join("jobs");
        fs::create_dir_all(&jobs_dir).context("Failed to create jobs directory")?;

        Ok(Self { jobs_dir })
    }

    /// Generate a unique job ID
    pub fn generate_job_id(task_name: &str) -> String {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let random: u32 = rand_id();
        format!("{}-{}-{:06x}", timestamp, sanitize_name(task_name), random)
    }

    /// Get the path for job metadata file
    pub fn job_path(&self, job_id: &str) -> PathBuf {
        self.jobs_dir.join(format!("{}.json", job_id))
    }

    /// Get the path for job log file
    pub fn log_path(&self, job_id: &str) -> PathBuf {
        self.jobs_dir.join(format!("{}.log", job_id))
    }

    /// Register a new job
    pub fn register(
        &self,
        job_id: &str,
        pid: u32,
        task_name: &str,
        transcript_path: &Path,
        job_type: JobType,
    ) -> Result<JobInfo> {
        let info = JobInfo {
            id: job_id.to_string(),
            pid,
            task_name: task_name.to_string(),
            transcript_path: transcript_path.to_path_buf(),
            started_at: Local::now(),
            finished_at: None,
            status: JobStatus::Running,
            job_type,
        };

        self.save_job(&info)?;
        Ok(info)
    }

    /// Save job info to disk
    fn save_job(&self, info: &JobInfo) -> Result<()> {
        let path = self.job_path(&info.id);
        let content = serde_json::to_string_pretty(info)?;
        fs::write(&path, content).context("Failed to save job info")?;
        Ok(())
    }

    /// Load job info from disk
    pub fn load_job(&self, job_id: &str) -> Result<JobInfo> {
        let path = self.job_path(job_id);
        let content = fs::read_to_string(&path).context("Failed to read job info")?;
        let info: JobInfo = serde_json::from_str(&content).context("Failed to parse job info")?;
        Ok(info)
    }

    /// Mark a job as completed
    pub fn mark_completed(&self, job_id: &str) -> Result<()> {
        let mut info = self.load_job(job_id)?;
        info.status = JobStatus::Completed;
        info.finished_at = Some(Local::now());
        self.save_job(&info)
    }

    /// Mark a job as failed
    pub fn mark_failed(&self, job_id: &str, error: &str) -> Result<()> {
        let mut info = self.load_job(job_id)?;
        info.status = JobStatus::Failed {
            error: error.to_string(),
        };
        info.finished_at = Some(Local::now());
        self.save_job(&info)
    }

    /// List all jobs, optionally filtering by status
    pub fn list(&self, include_completed: bool) -> Result<Vec<JobInfo>> {
        let mut jobs = vec![];

        for entry in fs::read_dir(&self.jobs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut info) = serde_json::from_str::<JobInfo>(&content) {
                        // Update status if process died unexpectedly
                        if info.status == JobStatus::Running && !info.is_alive() {
                            info.status = JobStatus::Failed {
                                error: "Process terminated unexpectedly".to_string(),
                            };
                            info.finished_at = Some(Local::now());
                            let _ = self.save_job(&info);
                        }

                        if include_completed || info.status == JobStatus::Running {
                            jobs.push(info);
                        }
                    }
                }
            }
        }

        // Sort by start time, newest first
        jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(jobs)
    }

    /// Get log content for a job
    pub fn read_log(&self, job_id: &str, tail_lines: Option<usize>) -> Result<String> {
        let path = self.log_path(job_id);
        let content = fs::read_to_string(&path).context("Failed to read log file")?;

        match tail_lines {
            Some(n) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = lines.len().saturating_sub(n);
                Ok(lines[start..].join("\n"))
            }
            None => Ok(content),
        }
    }

    /// Kill a running job
    pub fn kill(&self, job_id: &str) -> Result<bool> {
        let info = self.load_job(job_id)?;

        if info.status != JobStatus::Running {
            return Ok(false);
        }

        let killed = kill_process(info.pid);

        if killed {
            self.mark_failed(job_id, "Killed by user")?;
        }

        Ok(killed)
    }

    /// Cleanup old jobs
    pub fn cleanup(&self, keep_days: u32) -> Result<usize> {
        let cutoff = Local::now() - chrono::Duration::days(keep_days as i64);
        let mut removed = 0;

        for entry in fs::read_dir(&self.jobs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(info) = serde_json::from_str::<JobInfo>(&content) {
                        // Only remove completed/failed jobs older than cutoff
                        if info.status != JobStatus::Running && info.started_at < cutoff {
                            // Remove both json and log files
                            let _ = fs::remove_file(&path);
                            let _ = fs::remove_file(self.log_path(&info.id));
                            removed += 1;
                        }
                    }
                }
            }
        }

        Ok(removed)
    }

    /// Create a bounded log file for a job
    pub fn create_log_file(&self, job_id: &str) -> Result<std::fs::File> {
        let path = self.log_path(job_id);
        let file = fs::File::create(&path).context("Failed to create log file")?;
        Ok(file)
    }

    /// Truncate log file if it exceeds max size
    pub fn truncate_log_if_needed(&self, job_id: &str) -> Result<()> {
        let path = self.log_path(job_id);

        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.len() > MAX_LOG_SIZE {
                // Read file, keep last half
                let content = fs::read_to_string(&path)?;
                let lines: Vec<&str> = content.lines().collect();
                let keep_from = lines.len() / 2;
                let truncated = format!(
                    "[... log truncated, showing last {} lines ...]\n{}",
                    lines.len() - keep_from,
                    lines[keep_from..].join("\n")
                );
                fs::write(&path, truncated)?;
            }
        }

        Ok(())
    }
}

/// Sanitize task name for use in job ID
fn sanitize_name(name: &str) -> String {
    name.chars()
        .take(20)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// Generate a simple random ID component
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Mix nanoseconds for some randomness
    (duration.as_nanos() as u32) ^ (duration.subsec_nanos())
}

/// Check if a process is alive
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    // kill with signal 0 checks if process exists without sending a signal
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_alive(_pid: u32) -> bool {
    // On non-Unix, assume alive (can be improved with platform-specific APIs)
    true
}

/// Kill a process
#[cfg(unix)]
fn kill_process(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, libc::SIGTERM) == 0 }
}

#[cfg(not(unix))]
fn kill_process(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("my-project"), "my-project");
        assert_eq!(sanitize_name("My Project!"), "my-project-");
        assert_eq!(
            sanitize_name("very-long-project-name-that-exceeds-limit"),
            "very-long-project-na"
        );
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(format!("{}", JobStatus::Running), "Running");
        assert_eq!(format!("{}", JobStatus::Completed), "Completed");
        assert_eq!(
            format!(
                "{}",
                JobStatus::Failed {
                    error: "test".into()
                }
            ),
            "Failed: test"
        );
    }
}
