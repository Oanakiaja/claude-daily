use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::{Arc, RwLock};

use crate::archive::ArchiveManager;
use crate::config::{save_config, Config};
use crate::jobs::JobManager;
use crate::summarizer::Prompts;

use super::dto::*;

/// Shared application state
pub struct AppState {
    pub config: RwLock<Config>,
}

/// List all available dates
pub async fn list_dates(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    let manager = ArchiveManager::new(config);

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
    let config = state.config.read().unwrap().clone();
    let manager = ArchiveManager::new(config);

    match manager.read_daily_summary(&date) {
        Ok(content) => {
            let file_path = manager.daily_summary_path(&date);
            let mut summary = parse_daily_summary(&date, &content);
            summary.file_path = file_path.to_string_lossy().to_string();
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
    let config = state.config.read().unwrap().clone();
    let manager = ArchiveManager::new(config);

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
    let config = state.config.read().unwrap().clone();
    let manager = ArchiveManager::new(config);

    match manager.read_session(&date, &name) {
        Ok(content) => {
            let metadata = extract_session_metadata(&content);
            let file_path = manager.session_archive_path(&date, &name);
            let detail = SessionDetailDto {
                name,
                content,
                metadata,
                file_path: file_path.to_string_lossy().to_string(),
            };
            Json(ApiResponse::success(detail))
        }
        Err(e) => Json(ApiResponse::<SessionDetailDto>::error(e.to_string())),
    }
}

/// List all jobs
pub async fn list_jobs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = state.config.read().unwrap();
    match JobManager::new(&config) {
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
    let config = state.config.read().unwrap();
    match JobManager::new(&config) {
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
    let config = state.config.read().unwrap();
    match JobManager::new(&config) {
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
    let config = state.config.read().unwrap();
    match JobManager::new(&config) {
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

/// Trigger digest for a specific date
pub async fn trigger_digest(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    let manager = ArchiveManager::new(config);

    // Check if there are sessions to digest
    match manager.list_sessions(&date) {
        Ok(sessions) => {
            if sessions.is_empty() {
                return Json(ApiResponse::<DigestResponse>::error(format!(
                    "No sessions found for {}",
                    date
                )));
            }

            // Spawn background digest process
            let exe = match std::env::current_exe() {
                Ok(e) => e,
                Err(e) => {
                    return Json(ApiResponse::<DigestResponse>::error(format!(
                        "Failed to get executable: {}",
                        e
                    )));
                }
            };

            match std::process::Command::new(&exe)
                .args(["digest", "--date", &date])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => Json(ApiResponse::success(DigestResponse {
                    message: format!("Digest started for {} ({} sessions)", date, sessions.len()),
                    session_count: sessions.len(),
                })),
                Err(e) => Json(ApiResponse::<DigestResponse>::error(format!(
                    "Failed to start digest: {}",
                    e
                ))),
            }
        }
        Err(e) => Json(ApiResponse::<DigestResponse>::error(e.to_string())),
    }
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Get current configuration
pub async fn get_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = state.config.read().unwrap();
    let config_dto = ConfigDto {
        storage_path: config.storage.path.to_string_lossy().to_string(),
        model: config.summarization.model.clone(),
        summary_language: config.summarization.summary_language.clone(),
        enable_daily_summary: config.summarization.enable_daily_summary,
        enable_extraction_hints: config.summarization.enable_extraction_hints,
        auto_digest_enabled: config.summarization.auto_digest_enabled,
        digest_time: config.summarization.digest_time.clone(),
        author: config.archive.author.clone(),
        prompt_templates: PromptTemplatesDto {
            session_summary: config.prompt_templates.session_summary.clone(),
            daily_summary: config.prompt_templates.daily_summary.clone(),
            skill_extract: config.prompt_templates.skill_extract.clone(),
            command_extract: config.prompt_templates.command_extract.clone(),
        },
        auto_summarize_enabled: config.summarization.auto_summarize_enabled,
        auto_summarize_on_show: config.summarization.auto_summarize_on_show,
        auto_summarize_inactive_minutes: config.summarization.auto_summarize_inactive_minutes,
    };
    Json(ApiResponse::success(config_dto))
}

/// Update configuration
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfigUpdateRequest>,
) -> impl IntoResponse {
    let mut config = state.config.write().unwrap();

    // Update fields if provided
    if let Some(lang) = req.summary_language {
        if lang == "en" || lang == "zh" {
            config.summarization.summary_language = lang;
        } else {
            return Json(ApiResponse::<ConfigDto>::error(
                "Invalid language. Must be 'en' or 'zh'",
            ));
        }
    }
    if let Some(model) = req.model {
        if model == "sonnet" || model == "haiku" {
            config.summarization.model = model;
        } else {
            return Json(ApiResponse::<ConfigDto>::error(
                "Invalid model. Must be 'sonnet' or 'haiku'",
            ));
        }
    }
    if let Some(enable) = req.enable_daily_summary {
        config.summarization.enable_daily_summary = enable;
    }
    if let Some(enable) = req.enable_extraction_hints {
        config.summarization.enable_extraction_hints = enable;
    }
    if let Some(enable) = req.auto_digest_enabled {
        config.summarization.auto_digest_enabled = enable;
    }
    if let Some(time) = req.digest_time {
        // Validate time format
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                if h < 24 && m < 60 {
                    config.summarization.digest_time = time;
                }
            }
        }
    }
    if let Some(author) = req.author {
        config.archive.author = if author.is_empty() {
            None
        } else {
            Some(author)
        };
    }

    // Update prompt templates if provided
    if let Some(templates) = req.prompt_templates {
        if let Some(t) = templates.session_summary {
            config.prompt_templates.session_summary = if t.is_empty() { None } else { Some(t) };
        }
        if let Some(t) = templates.daily_summary {
            config.prompt_templates.daily_summary = if t.is_empty() { None } else { Some(t) };
        }
        if let Some(t) = templates.skill_extract {
            config.prompt_templates.skill_extract = if t.is_empty() { None } else { Some(t) };
        }
        if let Some(t) = templates.command_extract {
            config.prompt_templates.command_extract = if t.is_empty() { None } else { Some(t) };
        }
    }

    // Update auto-summarize settings
    if let Some(enable) = req.auto_summarize_enabled {
        config.summarization.auto_summarize_enabled = enable;
    }
    if let Some(on_show) = req.auto_summarize_on_show {
        config.summarization.auto_summarize_on_show = on_show;
    }
    if let Some(minutes) = req.auto_summarize_inactive_minutes {
        // Validate range: 5 minutes to 8 hours
        if (5..=480).contains(&minutes) {
            config.summarization.auto_summarize_inactive_minutes = minutes;
        }
    }

    // Save config to file
    if let Err(e) = save_config(&config) {
        return Json(ApiResponse::<ConfigDto>::error(format!(
            "Failed to save config: {}",
            e
        )));
    }

    // Return updated config
    let config_dto = ConfigDto {
        storage_path: config.storage.path.to_string_lossy().to_string(),
        model: config.summarization.model.clone(),
        summary_language: config.summarization.summary_language.clone(),
        enable_daily_summary: config.summarization.enable_daily_summary,
        enable_extraction_hints: config.summarization.enable_extraction_hints,
        auto_digest_enabled: config.summarization.auto_digest_enabled,
        digest_time: config.summarization.digest_time.clone(),
        author: config.archive.author.clone(),
        prompt_templates: PromptTemplatesDto {
            session_summary: config.prompt_templates.session_summary.clone(),
            daily_summary: config.prompt_templates.daily_summary.clone(),
            skill_extract: config.prompt_templates.skill_extract.clone(),
            command_extract: config.prompt_templates.command_extract.clone(),
        },
        auto_summarize_enabled: config.summarization.auto_summarize_enabled,
        auto_summarize_on_show: config.summarization.auto_summarize_on_show,
        auto_summarize_inactive_minutes: config.summarization.auto_summarize_inactive_minutes,
    };
    Json(ApiResponse::success(config_dto))
}

/// Get default prompt templates
pub async fn get_default_templates() -> impl IntoResponse {
    let defaults = DefaultTemplatesDto {
        session_summary_en: Prompts::default_session_summary_template("en").to_string(),
        session_summary_zh: Prompts::default_session_summary_template("zh").to_string(),
        daily_summary_en: Prompts::default_daily_summary_template("en").to_string(),
        daily_summary_zh: Prompts::default_daily_summary_template("zh").to_string(),
        skill_extract_en: Prompts::default_skill_extract_template("en").to_string(),
        skill_extract_zh: Prompts::default_skill_extract_template("zh").to_string(),
        command_extract_en: Prompts::default_command_extract_template("en").to_string(),
        command_extract_zh: Prompts::default_command_extract_template("zh").to_string(),
    };
    Json(ApiResponse::success(defaults))
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
        file_path: String::new(), // Will be set by caller
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
        if text.chars().count() > 200 {
            let truncated: String = text.chars().take(200).collect();
            format!("{}...", truncated)
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
                        _ => {}
                    }
                }
            }
        }
    }

    metadata
}
