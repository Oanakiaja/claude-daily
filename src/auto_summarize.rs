use anyhow::{Context, Result};
use chrono::{Local, NaiveTime};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::archive::ArchiveManager;
use crate::config::Config;
use crate::transcript::TranscriptParser;

/// Represents an unsummarized transcript that needs processing
#[derive(Debug, Clone)]
pub struct UnsummarizedTranscript {
    pub path: PathBuf,
    pub session_id: String,
    pub cwd: Option<PathBuf>,
}

/// Find all transcript files in Claude Code's projects directory
pub fn find_all_transcripts() -> Result<Vec<PathBuf>> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let projects_dir = home.join(".claude").join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut transcripts = Vec::new();

    // Recursively search for .jsonl files
    for entry in fs::read_dir(&projects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Search in subdirectories
            for sub_entry in fs::read_dir(&path)? {
                let sub_entry = sub_entry?;
                let sub_path = sub_entry.path();

                if sub_path.extension().is_some_and(|ext| ext == "jsonl") {
                    // Skip agent transcripts (temporary agent sessions)
                    if !sub_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|name| name.starts_with("agent-"))
                    {
                        transcripts.push(sub_path);
                    }
                }
            }
        }
    }

    Ok(transcripts)
}

/// Get all transcript paths that have been archived by reading frontmatter
fn get_archived_transcript_paths(config: &Config) -> Result<HashSet<String>> {
    let archive_manager = ArchiveManager::new(config.clone());
    let mut archived_paths = HashSet::new();

    // List all dates in the archive
    let dates = archive_manager.list_dates()?;

    for date in dates {
        let sessions = archive_manager.list_sessions(&date)?;
        for session in sessions {
            // Read the session file to extract transcript path from frontmatter
            if let Ok(content) = archive_manager.read_session(&date, &session) {
                // Extract transcript_path from frontmatter
                let mut in_frontmatter = false;
                for line in content.lines() {
                    if line.trim() == "---" {
                        if !in_frontmatter {
                            in_frontmatter = true;
                            continue;
                        } else {
                            // End of frontmatter
                            break;
                        }
                    }

                    if in_frontmatter && line.starts_with("transcript_path:") {
                        if let Some(path) = line.split(':').nth(1) {
                            let path_str = path.trim().trim_matches('"');
                            if path_str != "N/A" {
                                archived_paths.insert(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(archived_paths)
}

/// Check if a transcript has been recently modified (within the configured inactive threshold)
/// This helps avoid processing active sessions
fn is_transcript_active(path: &std::path::Path, inactive_minutes: u64) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                // Consider active if modified within the threshold
                return elapsed.as_secs() < inactive_minutes * 60;
            }
        }
    }
    false
}

/// Check if a transcript was modified yesterday
/// This helps limit auto-summarization to only yesterday's sessions
fn is_transcript_from_yesterday(path: &std::path::Path) -> bool {
    use chrono::{Duration, Local};

    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            let modified_dt = chrono::DateTime::<Local>::from(modified);
            let modified_date = modified_dt.date_naive();

            let now = Local::now();
            let yesterday = (now - Duration::days(1)).date_naive();
            let today = now.date_naive();

            // Accept both yesterday and today (for sessions that started yesterday but ended today)
            return modified_date == yesterday || modified_date == today;
        }
    }
    false
}

/// Find transcripts that have not been summarized yet
///
/// This function now uses transcript_path from session.md frontmatter for accurate tracking.
/// It also applies safety measures:
/// 1. Only processes transcripts from yesterday or today (to avoid processing too many old files)
/// 2. Only processes transcripts that haven't been modified in the last 2 hours (likely inactive)
/// 3. Limits to MAX_AUTO_SUMMARIZE to prevent fork bomb
pub fn find_unsummarized_transcripts(config: &Config) -> Result<Vec<UnsummarizedTranscript>> {
    let all_transcripts = find_all_transcripts()?;
    let archived_paths = get_archived_transcript_paths(config)?;

    let mut unsummarized = Vec::new();
    const MAX_AUTO_SUMMARIZE: usize = 3; // Conservative limit to prevent fork bomb

    for transcript_path in all_transcripts {
        // IMPORTANT: Only process transcripts from yesterday or today
        // This prevents processing too many old files when switching directories
        if !is_transcript_from_yesterday(&transcript_path) {
            continue;
        }

        // Skip if transcript is still active (modified within the configured threshold)
        if is_transcript_active(
            &transcript_path,
            config.summarization.auto_summarize_inactive_minutes,
        ) {
            continue;
        }

        // Check if already archived by exact path matching
        let path_str = transcript_path.to_string_lossy().to_string();
        if archived_paths.contains(&path_str) {
            continue;
        }

        // Check if the transcript file is empty or invalid
        let data = match TranscriptParser::parse(&transcript_path) {
            Ok(data) => data,
            Err(_) => continue,
        };

        // Skip empty transcripts
        if data.is_empty() {
            continue;
        }

        // Extract session ID from file name
        let session_id = transcript_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        unsummarized.push(UnsummarizedTranscript {
            path: transcript_path.clone(),
            session_id,
            cwd: None, // CWD is not available from transcript files
        });

        // Conservative limit to prevent fork bomb
        if unsummarized.len() >= MAX_AUTO_SUMMARIZE {
            break;
        }
    }

    Ok(unsummarized)
}

/// Check if auto-summarization should be triggered on `daily show`
///
/// Returns true if:
/// 1. auto_summarize_enabled is true (master switch)
/// 2. auto_summarize_on_show is true
///
/// This bypasses time-based checks and triggers on every `daily show` invocation.
pub fn should_trigger_auto_summarize_on_show(config: &Config) -> bool {
    config.summarization.auto_summarize_enabled && config.summarization.auto_summarize_on_show
}

/// Check if auto-summarization should be triggered (time-based)
///
/// Returns true if:
/// 1. auto_summarize_enabled is true
/// 2. Current time is after the configured trigger time (e.g., 06:00)
/// 3. Last check was NOT today after the trigger time
///
/// Note: This function only controls WHEN to check, not WHICH transcripts to process.
/// The actual filtering (only yesterday's transcripts) is done in find_unsummarized_transcripts()
pub fn should_trigger_auto_summarize(config: &Config) -> Result<bool> {
    if !config.summarization.auto_summarize_enabled {
        return Ok(false);
    }

    let now = Local::now();
    let today_date = now.format("%Y-%m-%d").to_string();

    // Parse trigger time
    let trigger_time =
        NaiveTime::parse_from_str(&config.summarization.auto_summarize_time, "%H:%M")
            .context("Invalid auto_summarize_time format")?;

    // Check if current time is after trigger time
    let current_time = now.time();
    if current_time < trigger_time {
        return Ok(false);
    }

    // Check last check time
    if let Some(last_check_str) = &config.summarization.last_auto_summarize_check {
        if let Ok(last_check) = chrono::DateTime::parse_from_rfc3339(last_check_str) {
            let last_check_date = last_check.format("%Y-%m-%d").to_string();

            // If last check was today and after trigger time, don't trigger again
            if last_check_date == today_date {
                let last_check_time = last_check.time();
                if last_check_time >= trigger_time {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_trigger_auto_summarize_disabled() {
        let mut config = Config::default();
        config.summarization.auto_summarize_enabled = false;

        assert!(!should_trigger_auto_summarize(&config).unwrap());
    }

    #[test]
    fn test_should_trigger_auto_summarize_no_last_check() {
        let mut config = Config::default();
        config.summarization.auto_summarize_enabled = true;
        config.summarization.auto_summarize_time = "00:00".to_string(); // Always trigger after midnight

        assert!(should_trigger_auto_summarize(&config).unwrap());
    }
}
