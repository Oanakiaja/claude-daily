use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::{Arc, RwLock};

use crate::archive::ArchiveManager;
use crate::config::{save_config, Config};
use crate::insights::collector::InsightsData;
use crate::insights::daily::DateInsights;
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

/// Get insights data
pub async fn get_insights(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    let days: usize = params
        .get("days")
        .and_then(|d| d.parse().ok())
        .unwrap_or(30);

    match InsightsData::collect(&config, Some(days)) {
        Ok(data) => {
            let dto = InsightsDto {
                total_days: data.total_days,
                total_sessions: data.total_sessions,
                daily_stats: data
                    .daily_stats
                    .into_iter()
                    .map(|s| DailyStatDto {
                        date: s.date,
                        session_count: s.session_count,
                        has_digest: s.has_digest,
                    })
                    .collect(),
                goal_distribution: data
                    .goal_distribution
                    .into_iter()
                    .map(|c| CategoryCountDto {
                        name: c.name,
                        count: c.count,
                    })
                    .collect(),
                friction_distribution: data
                    .friction_distribution
                    .into_iter()
                    .map(|c| CategoryCountDto {
                        name: c.name,
                        count: c.count,
                    })
                    .collect(),
                satisfaction_distribution: data
                    .satisfaction_distribution
                    .into_iter()
                    .map(|c| CategoryCountDto {
                        name: c.name,
                        count: c.count,
                    })
                    .collect(),
                language_distribution: data
                    .language_distribution
                    .into_iter()
                    .map(|c| CategoryCountDto {
                        name: c.name,
                        count: c.count,
                    })
                    .collect(),
                session_type_distribution: data
                    .session_type_distribution
                    .into_iter()
                    .map(|c| CategoryCountDto {
                        name: c.name,
                        count: c.count,
                    })
                    .collect(),
                session_details: data
                    .session_details
                    .into_iter()
                    .map(|s| SessionInsightDto {
                        session_id: s.session_id,
                        date: s.date,
                        session_name: s.session_name,
                        brief_summary: s.brief_summary,
                        outcome: s.outcome,
                        goal_categories: s.goal_categories,
                        friction_types: s.friction_types,
                        friction_detail: s.friction_detail,
                        satisfaction: s.satisfaction,
                        claude_helpfulness: s.claude_helpfulness,
                        session_type: s.session_type,
                    })
                    .collect(),
                trends: data.trends.map(|t| TrendDto {
                    period_label: t.period_label,
                    comparison_label: t.comparison_label,
                    current_sessions: t.current_sessions,
                    previous_sessions: t.previous_sessions,
                    sessions_change_pct: t.sessions_change_pct,
                    current_friction_rate: t.current_friction_rate,
                    previous_friction_rate: t.previous_friction_rate,
                    friction_change_pct: t.friction_change_pct,
                    current_success_rate: t.current_success_rate,
                    previous_success_rate: t.previous_success_rate,
                    success_change_pct: t.success_change_pct,
                    current_satisfaction_score: t.current_satisfaction_score,
                    previous_satisfaction_score: t.previous_satisfaction_score,
                    satisfaction_change_pct: t.satisfaction_change_pct,
                    weekly_stats: t
                        .weekly_stats
                        .into_iter()
                        .map(|w| WeeklyStatDto {
                            week_label: w.week_label,
                            session_count: w.session_count,
                            friction_rate: w.friction_rate,
                            success_rate: w.success_rate,
                        })
                        .collect(),
                }),
            };
            Json(ApiResponse::success(dto))
        }
        Err(e) => Json(ApiResponse::<InsightsDto>::error(e.to_string())),
    }
}

/// Get per-day insights combining session facet data
pub async fn get_date_insights(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();

    match DateInsights::collect(&date, &config) {
        Ok(data) => {
            let dto = DateInsightsDto {
                sessions: data
                    .sessions
                    .into_iter()
                    .map(|s| DateSessionInsightDto {
                        name: s.name,
                        session_id: s.session_id,
                        brief_summary: s.brief_summary,
                        outcome: s.outcome,
                        goal_categories: s.goal_categories,
                        friction_types: s.friction_types,
                        friction_detail: s.friction_detail,
                        satisfaction: s.satisfaction,
                        claude_helpfulness: s.claude_helpfulness,
                    })
                    .collect(),
                day_summary: DayInsightSummaryDto {
                    total_sessions: data.day_summary.total_sessions,
                    sessions_with_friction: data.day_summary.sessions_with_friction,
                    overall_satisfaction: data.day_summary.overall_satisfaction,
                    top_goals: data.day_summary.top_goals,
                    top_frictions: data.day_summary.top_frictions,
                    recommendations: data.day_summary.recommendations,
                },
            };
            Json(ApiResponse::success(dto))
        }
        Err(e) => Json(ApiResponse::<DateInsightsDto>::error(e.to_string())),
    }
}

/// Get session conversation (transcript parsed into chat messages)
pub async fn get_session_conversation(
    State(state): State<Arc<AppState>>,
    Path((date, name)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let config = state.config.read().unwrap().clone();
    let manager = ArchiveManager::new(config);

    // Read session markdown to extract transcript_path from frontmatter
    let transcript_path = match manager.read_session(&date, &name) {
        Ok(content) => extract_transcript_path(&content),
        Err(e) => {
            return Json(ApiResponse::<ConversationDto>::error(format!(
                "Failed to read session: {}",
                e
            )))
        }
    };

    let transcript_path = match transcript_path {
        Some(p) => p,
        None => {
            return Json(ApiResponse::success(ConversationDto {
                messages: vec![],
                total_entries: 0,
                has_transcript: false,
                page: 0,
                page_size: 0,
                has_more: false,
            }))
        }
    };

    // Check if transcript file exists
    let path = std::path::Path::new(&transcript_path);
    if !path.exists() {
        return Json(ApiResponse::success(ConversationDto {
            messages: vec![],
            total_entries: 0,
            has_transcript: false,
            page: 0,
            page_size: 0,
            has_more: false,
        }));
    }

    let page: usize = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(0);
    let page_size: usize = params
        .get("page_size")
        .and_then(|p| p.parse().ok())
        .unwrap_or(50);

    match parse_transcript_to_conversation(&transcript_path, page, page_size) {
        Ok(dto) => Json(ApiResponse::success(dto)),
        Err(e) => Json(ApiResponse::<ConversationDto>::error(format!(
            "Failed to parse transcript: {}",
            e
        ))),
    }
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

/// Extract transcript_path from session markdown YAML frontmatter
fn extract_transcript_path(content: &str) -> Option<String> {
    if let Some(stripped) = content.strip_prefix("---\n") {
        if let Some(end) = stripped.find("\n---") {
            let frontmatter = &stripped[..end];
            for line in frontmatter.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    if key == "transcript_path" {
                        let value = value.trim().trim_matches('"');
                        if value != "N/A" && !value.is_empty() {
                            return Some(value.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse JSONL transcript file into paginated ConversationDto
fn parse_transcript_to_conversation(
    path: &str,
    page: usize,
    page_size: usize,
) -> anyhow::Result<ConversationDto> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut conversation_messages: Vec<ConversationMessage> = Vec::new();
    // Collect tool results keyed by tool_use_id for later pairing
    let mut tool_results: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Buffer for merging consecutive assistant entries
    let mut current_assistant_blocks: Vec<ConversationContentBlock> = Vec::new();
    let mut current_assistant_timestamp: Option<String> = None;

    let flush_assistant = |blocks: &mut Vec<ConversationContentBlock>,
                           ts: &mut Option<String>,
                           messages: &mut Vec<ConversationMessage>| {
        if !blocks.is_empty() {
            messages.push(ConversationMessage {
                role: "assistant".to_string(),
                content: std::mem::take(blocks),
                timestamp: ts.take(),
            });
        }
    };

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry_type = entry
            .get("type")
            .and_then(|v| v.as_str())
            .or_else(|| entry.get("role").and_then(|v| v.as_str()))
            .unwrap_or("");
        let timestamp = entry
            .get("timestamp")
            .and_then(|v| v.as_str())
            .map(String::from);

        match entry_type {
            "user" | "human" => {
                // Flush any buffered assistant blocks
                flush_assistant(
                    &mut current_assistant_blocks,
                    &mut current_assistant_timestamp,
                    &mut conversation_messages,
                );

                // Try new format: message.content
                let content_val = entry
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .or_else(|| entry.get("content"));

                match content_val {
                    Some(serde_json::Value::String(text)) => {
                        if !text.trim().is_empty() {
                            conversation_messages.push(ConversationMessage {
                                role: "user".to_string(),
                                content: vec![ConversationContentBlock::Text {
                                    text: text.clone(),
                                }],
                                timestamp,
                            });
                        }
                    }
                    Some(serde_json::Value::Array(arr)) => {
                        // Tool result blocks - collect for pairing
                        for block in arr {
                            let block_type =
                                block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if block_type == "tool_result" {
                                if let Some(tool_use_id) =
                                    block.get("tool_use_id").and_then(|v| v.as_str())
                                {
                                    // Extract text from content
                                    let result_text = extract_tool_result_text(block);
                                    tool_results.insert(tool_use_id.to_string(), result_text);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            "assistant" => {
                let content_val = entry
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .or_else(|| entry.get("content"));

                if current_assistant_timestamp.is_none() {
                    current_assistant_timestamp = timestamp;
                }

                match content_val {
                    Some(serde_json::Value::Array(blocks)) => {
                        for block in blocks {
                            let block_type =
                                block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            match block_type {
                                "text" => {
                                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                        if !text.trim().is_empty() {
                                            current_assistant_blocks.push(
                                                ConversationContentBlock::Text {
                                                    text: text.to_string(),
                                                },
                                            );
                                        }
                                    }
                                }
                                "tool_use" => {
                                    let tool_id = block
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let name = block
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let input = block
                                        .get("input")
                                        .cloned()
                                        .unwrap_or(serde_json::Value::Null);
                                    let input = truncate_json_value(input, 500);
                                    current_assistant_blocks.push(
                                        ConversationContentBlock::ToolUse {
                                            tool_use_id: tool_id,
                                            name,
                                            input,
                                        },
                                    );
                                }
                                // Skip thinking blocks
                                _ => {}
                            }
                        }
                    }
                    Some(serde_json::Value::String(text)) => {
                        // Old format: content as string
                        if !text.trim().is_empty() {
                            if current_assistant_timestamp.is_none() {
                                current_assistant_timestamp = entry
                                    .get("timestamp")
                                    .and_then(|v| v.as_str())
                                    .map(String::from);
                            }
                            current_assistant_blocks.push(ConversationContentBlock::Text {
                                text: text.to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            // Skip file-history-snapshot, TranscriptSummary, etc.
            _ => {}
        }
    }

    // Flush remaining assistant blocks
    flush_assistant(
        &mut current_assistant_blocks,
        &mut current_assistant_timestamp,
        &mut conversation_messages,
    );

    // Pair tool_results back into conversation as ToolResult blocks after their ToolUse
    let mut final_messages: Vec<ConversationMessage> = Vec::new();
    for msg in conversation_messages {
        if msg.role == "assistant" {
            let mut new_content: Vec<ConversationContentBlock> = Vec::new();
            for block in msg.content {
                new_content.push(block.clone());
                if let ConversationContentBlock::ToolUse {
                    ref tool_use_id, ..
                } = block
                {
                    if let Some(result) = tool_results.remove(tool_use_id) {
                        new_content.push(ConversationContentBlock::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: result,
                        });
                    }
                }
            }
            final_messages.push(ConversationMessage {
                role: msg.role,
                content: new_content,
                timestamp: msg.timestamp,
            });
        } else {
            final_messages.push(msg);
        }
    }

    let total_entries = final_messages.len();

    // Paginate
    let start = page * page_size;
    let end = (start + page_size).min(total_entries);
    let has_more = end < total_entries;
    let page_messages = if start < total_entries {
        final_messages[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(ConversationDto {
        messages: page_messages,
        total_entries,
        has_transcript: true,
        page,
        page_size,
        has_more,
    })
}

/// Extract text from a tool_result content block
fn extract_tool_result_text(block: &serde_json::Value) -> String {
    if let Some(content) = block.get("content") {
        match content {
            serde_json::Value::String(s) => {
                return truncate_text_str(s, 500);
            }
            serde_json::Value::Array(arr) => {
                let texts: Vec<&str> = arr
                    .iter()
                    .filter_map(|b| {
                        if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                            b.get("text").and_then(|t| t.as_str())
                        } else {
                            None
                        }
                    })
                    .collect();
                if !texts.is_empty() {
                    return truncate_text_str(&texts.join("\n"), 500);
                }
            }
            _ => {}
        }
    }
    String::new()
}

/// Truncate a string to max_len chars
fn truncate_text_str(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

/// Truncate deeply nested JSON string values
fn truncate_json_value(value: serde_json::Value, max_str_len: usize) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            if s.chars().count() > max_str_len {
                let truncated: String = s.chars().take(max_str_len).collect();
                serde_json::Value::String(format!("{}...", truncated))
            } else {
                serde_json::Value::String(s)
            }
        }
        serde_json::Value::Object(map) => {
            let truncated: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(k, v)| (k, truncate_json_value(v, max_str_len)))
                .collect();
            serde_json::Value::Object(truncated)
        }
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.into_iter()
                .map(|v| truncate_json_value(v, max_str_len))
                .collect(),
        ),
        other => other,
    }
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
