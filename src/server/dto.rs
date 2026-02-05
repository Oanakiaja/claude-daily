use serde::{Deserialize, Serialize};

use crate::jobs::{JobInfo, JobStatus, JobType};

/// Generic API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Date info for listing
#[derive(Serialize)]
pub struct DateInfo {
    pub date: String,
    pub session_count: usize,
    pub has_digest: bool,
}

/// Brief session info for listing
#[derive(Serialize)]
pub struct SessionBrief {
    pub name: String,
    pub title: String,
    pub summary_preview: String,
}

/// Daily summary DTO
#[derive(Serialize)]
pub struct DailySummaryDto {
    pub date: String,
    pub overview: String,
    pub session_count: usize,
    pub sessions: Vec<String>,
    pub insights: Option<String>,
    pub skills: Option<String>,
    pub commands: Option<String>,
    pub reflections: Option<String>,
    pub tomorrow_focus: Option<String>,
    pub raw_content: String,
    pub file_path: String,
}

/// Session detail DTO
#[derive(Serialize)]
pub struct SessionDetailDto {
    pub name: String,
    pub content: String,
    pub metadata: SessionMetadata,
    pub file_path: String,
}

/// Session metadata extracted from frontmatter
#[derive(Serialize, Default)]
pub struct SessionMetadata {
    pub title: String,
    pub date: String,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub duration: Option<String>,
}

/// Job DTO for API responses
#[derive(Serialize, Deserialize, Clone)]
pub struct JobDto {
    pub id: String,
    pub pid: u32,
    pub task_name: String,
    pub status: String,
    pub status_type: String,
    pub job_type: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub elapsed: String,
}

impl From<JobInfo> for JobDto {
    fn from(info: JobInfo) -> Self {
        let (status, status_type) = match &info.status {
            JobStatus::Running => ("Running".to_string(), "running".to_string()),
            JobStatus::Completed => ("Completed".to_string(), "completed".to_string()),
            JobStatus::Failed { error } => (format!("Failed: {}", error), "failed".to_string()),
        };

        let job_type = match &info.job_type {
            JobType::SessionEnd => "session_end".to_string(),
            JobType::AutoSummarize => "auto_summarize".to_string(),
            JobType::Manual => "manual".to_string(),
        };

        // Compute elapsed before moving fields
        let elapsed = info.elapsed_human();
        let started_at = info.started_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let finished_at = info
            .finished_at
            .map(|t: chrono::DateTime<chrono::Local>| t.format("%Y-%m-%d %H:%M:%S").to_string());

        Self {
            id: info.id,
            pid: info.pid,
            task_name: info.task_name,
            status,
            status_type,
            job_type,
            started_at,
            finished_at,
            elapsed,
        }
    }
}

/// Job log response
#[derive(Serialize)]
pub struct JobLogDto {
    pub id: String,
    pub content: String,
}

/// Digest trigger response
#[derive(Serialize)]
pub struct DigestResponse {
    pub message: String,
    pub session_count: usize,
}

/// WebSocket message types
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    JobUpdated(JobDto),
    NewSession { date: String, name: String },
    DigestCompleted { date: String },
    Connected,
}

/// Config DTO for API responses
#[derive(Serialize)]
pub struct ConfigDto {
    pub storage_path: String,
    pub model: String,
    pub summary_language: String,
    pub enable_daily_summary: bool,
    pub enable_extraction_hints: bool,
    pub auto_digest_enabled: bool,
    pub digest_time: String,
    pub author: Option<String>,
    pub prompt_templates: PromptTemplatesDto,
    pub auto_summarize_enabled: bool,
    pub auto_summarize_on_show: bool,
    pub auto_summarize_inactive_minutes: u64,
}

/// Config update request
#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub summary_language: Option<String>,
    pub model: Option<String>,
    pub enable_daily_summary: Option<bool>,
    pub enable_extraction_hints: Option<bool>,
    pub auto_digest_enabled: Option<bool>,
    pub digest_time: Option<String>,
    pub author: Option<String>,
    pub prompt_templates: Option<PromptTemplatesUpdateRequest>,
    pub auto_summarize_enabled: Option<bool>,
    pub auto_summarize_on_show: Option<bool>,
    pub auto_summarize_inactive_minutes: Option<u64>,
}

/// Prompt templates DTO for API responses
#[derive(Serialize, Clone)]
pub struct PromptTemplatesDto {
    pub session_summary: Option<String>,
    pub daily_summary: Option<String>,
    pub skill_extract: Option<String>,
    pub command_extract: Option<String>,
}

/// Prompt templates update request
#[derive(Deserialize)]
pub struct PromptTemplatesUpdateRequest {
    pub session_summary: Option<String>,
    pub daily_summary: Option<String>,
    pub skill_extract: Option<String>,
    pub command_extract: Option<String>,
}

/// Default templates DTO for API responses
#[derive(Serialize)]
pub struct DefaultTemplatesDto {
    pub session_summary_en: String,
    pub session_summary_zh: String,
    pub daily_summary_en: String,
    pub daily_summary_zh: String,
    pub skill_extract_en: String,
    pub skill_extract_zh: String,
    pub command_extract_en: String,
    pub command_extract_zh: String,
}
