use anyhow::{Context, Result};
use chrono::{Duration, Local};
use std::process::{Command, Stdio};

use crate::archive::ArchiveManager;
use crate::config::load_config;
use crate::summarizer::SummarizerEngine;

/// Parse relative date string to actual date
fn parse_relative_date(relative: &str) -> Option<String> {
    match relative.to_lowercase().as_str() {
        "yest" | "yesterday" => {
            let yesterday = Local::now() - Duration::days(1);
            Some(yesterday.format("%Y-%m-%d").to_string())
        }
        "today" => Some(Local::now().format("%Y-%m-%d").to_string()),
        _ => None,
    }
}

/// Run the digest command - generate daily summary from sessions
pub async fn run(
    relative_date: Option<String>,
    date: Option<String>,
    background: bool,
    force: bool,
) -> Result<()> {
    let config = load_config()?;

    // Determine target date: relative_date takes precedence, then --date, then today
    let target_date = if let Some(rel) = relative_date {
        parse_relative_date(&rel).unwrap_or_else(|| {
            eprintln!(
                "[daily] Unknown relative date '{}', using as literal date",
                rel
            );
            rel
        })
    } else {
        date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string())
    };

    let manager = ArchiveManager::new(config.clone());

    // Check if there are sessions to digest
    let sessions = manager.list_sessions(&target_date)?;

    // Handle no sessions case
    if sessions.is_empty() {
        if force {
            // Force mode: check if daily.md exists and regenerate
            if manager.read_daily_summary(&target_date).is_err() {
                eprintln!(
                    "[daily] No sessions and no existing daily.md for {}",
                    target_date
                );
                return Ok(());
            }
            eprintln!(
                "[daily] Force regenerating daily summary for {}...",
                target_date
            );
        } else {
            eprintln!("[daily] No sessions found for {}", target_date);
            return Ok(());
        }
    }

    if background {
        // Background mode: spawn detached process
        eprintln!(
            "[daily] Starting background digest for {} ({} sessions)",
            target_date,
            sessions.len()
        );

        let exe = std::env::current_exe().context("Failed to get current executable")?;

        let mut args = vec!["digest", "--date", &target_date];
        if force {
            args.push("--force");
        }

        Command::new(&exe)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn background digest process")?;

        eprintln!("[daily] Background digest started");
        return Ok(());
    }

    // Foreground mode: perform the digest
    if !sessions.is_empty() {
        eprintln!(
            "[daily] Digesting {} sessions for {}...",
            sessions.len(),
            target_date
        );
    }

    let engine = SummarizerEngine::new(config.clone());

    // Generate daily summary from all sessions (or regenerate if force mode)
    match engine.update_daily_summary(&target_date).await {
        Ok(summary) => {
            let summary_path = summary.save(&config)?;
            eprintln!("[daily] Daily summary created: {}", summary_path.display());
            // Session files are preserved for reference
        }
        Err(e) => {
            eprintln!("[daily] Error: Failed to create daily summary: {}", e);
            if !sessions.is_empty() {
                eprintln!("[daily] Session files preserved for retry");
            }
            return Err(e);
        }
    }

    eprintln!("[daily] Digest complete!");
    Ok(())
}
