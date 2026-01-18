use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::archive::ArchiveManager;
use crate::config::Config;
use crate::jobs::JobManager;

use super::dto::*;

/// Shared application state
pub struct AppState {
    pub config: Config,
}

/// List all available dates
pub async fn list_dates(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let manager = ArchiveManager::new(state.config.clone());

    match manager.list_dates() {
        Ok(dates) => {
            let date_infos: Vec<DateInfo> = dates
                .into_iter()
                .map(|date| {
                    let sessions = manager.list_sessions(&date).unwrap_or_default();
                    let has_digest = manager
                        .read_daily_summary(&date)
                        .map(|content| {
                            content.contains("## Overview")
                                && !content.contains("No sessions recorded yet")
                        })
                        .unwrap_or(false);

                    DateInfo {
                        date,
                        session_count: sessions.len(),
                        has_digest,
                    }
                })
                .collect();

            Json(ApiResponse::success(date_infos))
        }
        Err(e) => Json(ApiResponse::<Vec<DateInfo>>::error(e.to_string())),
    }
}

/// Get daily summary for a specific date
pub async fn get_daily_summary(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> impl IntoResponse {
    let manager = ArchiveManager::new(state.config.clone());

    match manager.read_daily_summary(&date) {
        Ok(content) => {
            let summary = parse_daily_summary(&date, &content);
            Json(ApiResponse::success(summary))
        }
        Err(e) => Json(ApiResponse::<DailySummaryDto>::error(e.to_string())),
    }
}

/// List sessions for a specific date
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> impl IntoResponse {
    let manager = ArchiveManager::new(state.config.clone());

    match manager.list_sessions(&date) {
        Ok(sessions) => {
            let session_briefs: Vec<SessionBrief> = sessions
                .into_iter()
                .filter_map(|name| {
                    manager.read_session(&date, &name).ok().map(|content| {
                        let (title, summary) = extract_session_preview(&content);
                        SessionBrief {
                            name,
                            title,
                            summary_preview: summary,
                        }
                    })
                })
                .collect();

            Json(ApiResponse::success(session_briefs))
        }
        Err(e) => Json(ApiResponse::<Vec<SessionBrief>>::error(e.to_string())),
    }
}

/// Get session details
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path((date, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let manager = ArchiveManager::new(state.config.clone());

    match manager.read_session(&date, &name) {
        Ok(content) => {
            let metadata = extract_session_metadata(&content);
            let detail = SessionDetailDto {
                name,
                content,
                metadata,
            };
            Json(ApiResponse::success(detail))
        }
        Err(e) => Json(ApiResponse::<SessionDetailDto>::error(e.to_string())),
    }
}

/// List all jobs
pub async fn list_jobs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match JobManager::new(&state.config) {
        Ok(manager) => match manager.list(true) {
            Ok(jobs) => {
                let job_dtos: Vec<JobDto> = jobs.into_iter().map(Into::into).collect();
                Json(ApiResponse::success(job_dtos))
            }
            Err(e) => Json(ApiResponse::<Vec<JobDto>>::error(e.to_string())),
        },
        Err(e) => Json(ApiResponse::<Vec<JobDto>>::error(e.to_string())),
    }
}

/// Get job details
pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match JobManager::new(&state.config) {
        Ok(manager) => match manager.load_job(&job_id) {
            Ok(job) => Json(ApiResponse::success(JobDto::from(job))),
            Err(e) => Json(ApiResponse::<JobDto>::error(e.to_string())),
        },
        Err(e) => Json(ApiResponse::<JobDto>::error(e.to_string())),
    }
}

/// Get job log
pub async fn get_job_log(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match JobManager::new(&state.config) {
        Ok(manager) => match manager.read_log(&job_id, None) {
            Ok(content) => Json(ApiResponse::success(JobLogDto {
                id: job_id,
                content,
            })),
            Err(e) => Json(ApiResponse::<JobLogDto>::error(e.to_string())),
        },
        Err(e) => Json(ApiResponse::<JobLogDto>::error(e.to_string())),
    }
}

/// Kill a job
pub async fn kill_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match JobManager::new(&state.config) {
        Ok(manager) => match manager.kill(&job_id) {
            Ok(killed) => {
                if killed {
                    Json(ApiResponse::success(serde_json::json!({ "killed": true })))
                } else {
                    Json(ApiResponse::error("Job not running or could not be killed"))
                }
            }
            Err(e) => Json(ApiResponse::<serde_json::Value>::error(e.to_string())),
        },
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(e.to_string())),
    }
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// Helper functions

fn parse_daily_summary(date: &str, content: &str) -> DailySummaryDto {
    let extract_section = |header: &str| -> Option<String> {
        let pattern = format!("## {}\n", header);
        if let Some(start) = content.find(&pattern) {
            let start = start + pattern.len();
            let end = content[start..]
                .find("\n## ")
                .map(|i| start + i)
                .unwrap_or(content.len());
            let section = content[start..end].trim().to_string();
            if section.is_empty() || section == "No sessions recorded yet." {
                None
            } else {
                Some(section)
            }
        } else {
            None
        }
    };

    // Extract session names from frontmatter or content
    let sessions: Vec<String> = if let Some(start) = content.find("sessions:") {
        let start = start + 9;
        let end = content[start..]
            .find("\n---")
            .or_else(|| content[start..].find("\ntags:"))
            .map(|i| start + i)
            .unwrap_or(content.len());
        content[start..end]
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                line.strip_prefix("- ")
                    .map(|stripped| stripped.trim_matches('"').to_string())
            })
            .collect()
    } else {
        Vec::new()
    };

    DailySummaryDto {
        date: date.to_string(),
        overview: extract_section("Overview").unwrap_or_default(),
        session_count: sessions.len(),
        sessions,
        insights: extract_section("Key Insights"),
        skills: extract_section("Skills"),
        commands: extract_section("Commands"),
        reflections: extract_section("Reflections"),
        tomorrow_focus: extract_section("Tomorrow's Focus"),
        raw_content: content.to_string(),
    }
}

fn extract_session_preview(content: &str) -> (String, String) {
    // Extract title from frontmatter or first heading
    let title = if let Some(start) = content.find("title:") {
        let start = start + 6;
        let end = content[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(content.len());
        content[start..end].trim().trim_matches('"').to_string()
    } else if let Some(start) = content.find("# ") {
        let start = start + 2;
        let end = content[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(content.len());
        content[start..end].trim().to_string()
    } else {
        "Untitled".to_string()
    };

    // Extract summary section preview
    let summary = if let Some(start) = content.find("## Summary\n") {
        let start = start + 11;
        let end = content[start..]
            .find("\n## ")
            .map(|i| start + i)
            .unwrap_or_else(|| (start + 300).min(content.len()));
        let text = content[start..end].trim();
        if text.len() > 200 {
            format!("{}...", &text[..200])
        } else {
            text.to_string()
        }
    } else {
        String::new()
    };

    (title, summary)
}

fn extract_session_metadata(content: &str) -> SessionMetadata {
    let mut metadata = SessionMetadata::default();

    // Parse YAML frontmatter
    if let Some(stripped) = content.strip_prefix("---\n") {
        if let Some(end) = stripped.find("\n---") {
            let frontmatter = &stripped[..end];
            for line in frontmatter.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    match key {
                        "title" => metadata.title = value.to_string(),
                        "date" => metadata.date = value.to_string(),
                        "session_id" => metadata.session_id = Some(value.to_string()),
                        "cwd" => metadata.cwd = Some(value.to_string()),
                        "git_branch" => metadata.git_branch = Some(value.to_string()),
                        "duration" => metadata.duration = Some(value.to_string()),
                        "tool_calls" => metadata.tool_calls = value.parse().ok(),
                        _ => {}
                    }
                }
            }
        }
    }

    metadata
}
